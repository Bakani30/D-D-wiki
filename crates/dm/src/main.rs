use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use rand::rngs::StdRng;
use rand::SeedableRng;

use dm_claude::PyAiClient;
use dm_core::check::{Check, D20Test};
use dm_core::combat::Encounter;
use dm_core::effect::Effect;
use dm_core::entity::{Creature, EntityId, Side};
use dm_core::intent::Intent;
use dm_core::scenario;
use dm_wiki::fixtures;
use dm_wiki::lmop::{extract_monsters, extract_scenes};
use dm_wiki::scene::{ManifestFile, SceneDef, SceneManifest};
use dm_wiki::session::SessionWriter;

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "dm", version, about = "D&D AI DM — Prototype 1 CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run a pre-built encounter scenario.
    Encounter {
        #[command(subcommand)]
        which: Scenario,
    },
    /// Generate a manifest.yaml from LMoP Markdown chapter + monster appendix.
    LmopInit {
        /// Story chapter Markdown (e.g. "ลูกศรของก็อบลิน...md").
        #[arg(long)]
        story: PathBuf,
        /// Monster appendix Markdown (ภาคผนวก B_ มอนสเตอร์.md).
        #[arg(long)]
        monsters: PathBuf,
        /// Output manifest YAML path.
        #[arg(long, default_value = "campaigns/lmop/scenes/manifest.yaml")]
        out: PathBuf,
        /// Also write a monsters-extracted.yaml fixture file alongside the manifest.
        #[arg(long)]
        dump_monsters: bool,
    },
}

#[derive(Subcommand)]
enum Scenario {
    /// Peril in Pinebrook — Encounter 2: Living Icicles.
    Pinebrook {
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long)]
        fixtures: Option<PathBuf>,
        #[arg(long, default_value = "campaigns/example")]
        out: PathBuf,
        /// Auto-play PCs — no prompts. Used for CI/seeded replay.
        #[arg(long)]
        auto: bool,
    },
    /// LMoP scene — AI-assisted via Python /api/suggest + /api/intent.
    Lmop {
        /// Path to the scene manifest YAML.
        #[arg(long)]
        manifest: PathBuf,
        /// Scene ID to start from (defaults to manifest entry scene).
        #[arg(long)]
        scene: Option<String>,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, default_value = "campaigns/lmop")]
        out: PathBuf,
        /// Python backend base URL.
        #[arg(long, default_value = "http://127.0.0.1:8000")]
        ai_url: String,
        /// Session number to write.
        #[arg(long, default_value = "1")]
        session: u32,
        /// Auto-play PCs — no prompts. Useful for smoke-testing.
        #[arg(long)]
        auto: bool,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        None => {
            println!("dm v{}", env!("CARGO_PKG_VERSION"));
            println!("Try: dm encounter pinebrook --seed 42");
            println!("     dm encounter lmop --manifest campaigns/lmop/scenes/manifest.yaml");
        }
        Some(Command::Encounter { which: Scenario::Pinebrook { seed, fixtures, out, auto } }) => {
            run_pinebrook(seed, fixtures, out, auto)?;
        }
        Some(Command::Encounter {
            which:
                Scenario::Lmop {
                    manifest,
                    scene,
                    seed,
                    out,
                    ai_url,
                    session,
                    auto,
                },
        }) => {
            // AI calls are async; build a minimal single-threaded runtime.
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?
                .block_on(run_lmop(manifest, scene, seed, out, ai_url, session, auto))?;
        }
        Some(Command::LmopInit { story, monsters, out, dump_monsters }) => {
            run_lmop_init(story, monsters, out, dump_monsters)?;
        }
    }
    Ok(())
}

// ── Pinebrook (unchanged sync loop) ──────────────────────────────────────────

fn run_pinebrook(
    seed: Option<u64>,
    fixtures_path: Option<PathBuf>,
    out: PathBuf,
    auto: bool,
) -> Result<()> {
    let creatures = match fixtures_path {
        Some(p) => fixtures::load_creatures(&p)
            .with_context(|| format!("loading fixture {}", p.display()))?,
        None => scenario::pinebrook::default_creatures(),
    };

    let seed = seed.unwrap_or_else(rand::random::<u64>);
    println!("Living Icicles encounter — seed {}", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let pcs: Vec<Creature> =
        creatures.iter().filter(|c| c.side == Side::Pcs).cloned().collect();
    let mut writer = SessionWriter::open_or_create(&out, 1, &pcs)?;
    println!("Session file: {}\n", writer.path().display());

    let mut enc = Encounter::new(creatures);
    emit(&mut writer, &enc.roll_initiative(&mut rng))?;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while !enc.is_ended() {
        let actor_id = match enc.current_actor() {
            Some(id) => id,
            None => break,
        };
        let actor = enc.find(actor_id).unwrap().clone();

        let targets: Vec<EntityId> = enc
            .creatures()
            .iter()
            .filter(|c| c.side != actor.side && !c.is_down())
            .map(|c| c.id)
            .collect();

        let target = if targets.is_empty() {
            None
        } else if auto || actor.side == Side::Enemies {
            Some(targets[0])
        } else {
            prompt_target(&enc, &actor, &targets, &mut lines)?
        };

        if let Some(t) = target {
            let fx = enc.resolve(
                Intent::Attack { attacker: actor_id, target: t, attack_idx: 0 },
                &mut rng,
            )?;
            for e in &fx {
                emit(&mut writer, e)?;
            }
            if enc.is_ended() {
                break;
            }
        }

        let fx = enc.resolve(Intent::EndTurn { actor: actor_id }, &mut rng)?;
        for e in &fx {
            emit(&mut writer, e)?;
        }
    }

    println!("\n--- Encounter complete ---");
    println!("Session saved to: {}", writer.path().display());
    Ok(())
}

// ── LMoP manifest initialiser ────────────────────────────────────────────────

fn run_lmop_init(
    story_path: PathBuf,
    monsters_path: PathBuf,
    out_path: PathBuf,
    dump_monsters: bool,
) -> Result<()> {
    let story_md = std::fs::read_to_string(&story_path)
        .with_context(|| format!("reading story file {}", story_path.display()))?;
    let monsters_md = std::fs::read_to_string(&monsters_path)
        .with_context(|| format!("reading monsters file {}", monsters_path.display()))?;

    let scenes = extract_scenes(&story_md);
    let monsters = extract_monsters(&monsters_md);

    println!("Extracted {} scene(s) from {}", scenes.len(), story_path.display());
    println!("Extracted {} monster(s) from {}", monsters.len(), monsters_path.display());

    // Build SceneDef list — entities are empty; user fills them from the
    // monsters-extracted.yaml dump or edits manually.
    let scene_defs: Vec<SceneDef> = scenes
        .iter()
        .enumerate()
        .map(|(i, s)| SceneDef {
            id: s.id.clone(),
            title: s.title.clone(),
            description: s.description.clone(),
            entities: vec![],
            next: scenes.get(i + 1).map(|n| n.id.clone()),
        })
        .collect();

    let entry = scene_defs.first().map(|s| s.id.clone()).unwrap_or_default();
    let manifest = ManifestFile { entry, scenes: scene_defs };

    // Write manifest YAML
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let yaml = serde_yaml::to_string(&manifest)?;
    std::fs::write(&out_path, &yaml)
        .with_context(|| format!("writing manifest to {}", out_path.display()))?;
    println!("Manifest written → {}", out_path.display());
    println!("  (entities: [] — copy stat blocks from monsters dump or fill manually)");

    // Optionally write a monster fixture dump
    if dump_monsters {
        let creatures: Vec<Creature> = monsters
            .iter()
            .enumerate()
            .map(|(i, m)| m.to_creature(101 + i as u32, 0, Side::Enemies))
            .collect();
        let monsters_out = out_path.with_file_name("monsters-extracted.yaml");
        let monsters_yaml = serde_yaml::to_string(&creatures)?;
        std::fs::write(&monsters_out, &monsters_yaml)
            .with_context(|| format!("writing monsters to {}", monsters_out.display()))?;
        println!("Monsters dump   → {}", monsters_out.display());
        for m in &monsters {
            println!(
                "  · {} ({}) AC {} HP {} init {:+}  {} attack(s)",
                m.name_thai, m.name_english, m.ac, m.hp, m.initiative_bonus, m.attacks.len()
            );
        }
    }

    Ok(())
}

// ── LMoP AI-assisted loop ─────────────────────────────────────────────────────

async fn run_lmop(
    manifest_path: PathBuf,
    scene_id: Option<String>,
    seed: Option<u64>,
    out: PathBuf,
    ai_url: String,
    session_number: u32,
    auto: bool,
) -> Result<()> {
    let manifest = SceneManifest::load(&manifest_path)
        .with_context(|| format!("loading manifest {}", manifest_path.display()))?;

    let scene_key = scene_id.as_deref().unwrap_or(manifest.entry_id());
    let scene = manifest
        .get(scene_key)
        .ok_or_else(|| anyhow!("scene '{}' not found in manifest", scene_key))?;

    println!("╔══════════════════════════════╗");
    println!("║  {}  ║", scene.title);
    println!("╚══════════════════════════════╝\n");
    println!("{}\n", scene.description);

    let ai = PyAiClient::new(&ai_url);

    let seed = seed.unwrap_or_else(rand::random::<u64>);
    println!("[seed {}]", seed);
    let mut rng = StdRng::seed_from_u64(seed);

    let pcs: Vec<Creature> =
        scene.entities.iter().filter(|c| c.side == Side::Pcs).cloned().collect();
    let mut writer = SessionWriter::open_or_create(&out, session_number, &pcs)?;
    println!("Session file: {}\n", writer.path().display());

    let mut enc = Encounter::new(scene.entities.clone());
    emit(&mut writer, &enc.roll_initiative(&mut rng))?;

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    while !enc.is_ended() {
        let actor_id = match enc.current_actor() {
            Some(id) => id,
            None => break,
        };
        let actor = enc.find(actor_id).unwrap().clone();

        let targets: Vec<EntityId> = enc
            .creatures()
            .iter()
            .filter(|c| c.side != actor.side && !c.is_down())
            .map(|c| c.id)
            .collect();

        if targets.is_empty() {
            // All opponents down — end the actor's turn; loop will detect
            // encounter end on the next pass.
            let fx = enc.resolve(Intent::EndTurn { actor: actor_id }, &mut rng)?;
            for e in &fx { emit(&mut writer, e)?; }
            continue;
        }

        if actor.side == Side::Enemies || auto {
            // Enemies always attack the first living PC.
            let fx = enc.resolve(
                Intent::Attack { attacker: actor_id, target: targets[0], attack_idx: 0 },
                &mut rng,
            )?;
            for e in &fx { emit(&mut writer, e)?; }
        } else {
            // ── PC turn: AI suggestions → classify → route ────────────────

            // 1. Ask Python for suggestions (optional — graceful offline).
            let suggestions = match ai.suggest(&scene.description).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("[AI offline: {}] — manual input only", e);
                    vec![]
                }
            };

            // 2. Display status panel + AI choices.
            print_hp_panel(&enc);
            println!("\n{}'s turn:", actor.name);

            if !suggestions.is_empty() {
                println!("╌ AI suggestions ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌");
                for (i, s) in suggestions.iter().enumerate() {
                    println!("  {}) {}", i + 1, s);
                }
                println!("╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌");
                println!("  0) Type your own action");
            }
            print!("> ");
            io::stdout().flush()?;

            let raw_input = match lines.next() {
                Some(Ok(l)) => l.trim().to_string(),
                _ => return Ok(()),
            };

            // 3. Map choice number → suggestion text (or use raw text directly).
            //    This is the bridge that solves blind spot 3: free-text Thai
            //    strings from /api/suggest become the *input* to /api/intent.
            let action_text = match raw_input.parse::<usize>() {
                Ok(n) if n >= 1 && n <= suggestions.len() => suggestions[n - 1].clone(),
                _ => raw_input,
            };

            // 4. Classify intent via Python MiniLM.
            let (label, confidence) = match ai.classify_intent(&action_text).await {
                Ok(r) => (r.intent, r.confidence),
                Err(e) => {
                    eprintln!("[classifier offline: {}] — treating as attack", e);
                    ("attack".to_string(), 0.0_f32)
                }
            };
            println!("[intent: {} ({:.0}%)]", label, confidence * 100.0);
            writer.append(&format!("[intent: {label} ({:.0}%)] \"{action_text}\"", confidence * 100.0))?;

            // 5. Route by label.
            //    "attack" → need target selection (blind spot 2: label alone
            //    is insufficient; we must prompt for the EntityId).
            match label.as_str() {
                "attack" => {
                    let target = prompt_target(&enc, &actor, &targets, &mut lines)?;
                    if let Some(t) = target {
                        let fx = enc.resolve(
                            Intent::Attack { attacker: actor_id, target: t, attack_idx: 0 },
                            &mut rng,
                        )?;
                        for e in &fx { emit(&mut writer, e)?; }
                    }
                }
                "ability_check" => {
                    // Phase 1 stub: Perception DC 15 using initiative_bonus as
                    // a proxy for Wisdom modifier. Phase 2 will use full skill list.
                    let check =
                        Check::new(D20Test::new(actor.initiative_bonus), 15);
                    let result = check.execute(&mut rng);
                    println!("Ability check — {}", result.narrative());
                    writer.append(&format!("- Ability check — {}", result.narrative()))?;
                }
                "cast_spell" => {
                    // Phase 2: look up spell in spell table, call /api/suggest
                    // for targeting, then resolve via Rules Engine.
                    println!("[{}] — spell resolution not yet implemented; ending turn.", label);
                }
                "roleplay" => {
                    // Phase 2: forward action_text to Narrative Agent (Haiku),
                    // get NPC response, run State Inference pass.
                    println!("[roleplay] — narration not yet implemented; ending turn.");
                }
                _ => {
                    println!("[unknown intent '{}'] — ending turn.", label);
                }
            }
        }

        if enc.is_ended() {
            break;
        }

        let fx = enc.resolve(Intent::EndTurn { actor: actor_id }, &mut rng)?;
        for e in &fx {
            emit(&mut writer, e)?;
        }
    }

    println!("\n═══ Scene complete ═══");
    println!("Session saved to: {}", writer.path().display());
    Ok(())
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn emit(writer: &mut SessionWriter, effect: &Effect) -> Result<()> {
    println!("{}", effect.narrative());
    writer.log_effect(effect)?;
    Ok(())
}

fn prompt_target<B: BufRead>(
    enc: &Encounter,
    actor: &Creature,
    targets: &[EntityId],
    lines: &mut std::io::Lines<B>,
) -> Result<Option<EntityId>> {
    println!("\nChoose a target for {}:", actor.name);
    for (i, t) in targets.iter().enumerate() {
        let c = enc.find(*t).unwrap();
        println!("  {}) {} (HP {}/{}, AC {})", i + 1, c.name, c.hp, c.max_hp, c.ac);
    }
    println!("  0) End turn");
    print!("> ");
    io::stdout().flush()?;

    let line = match lines.next() {
        Some(Ok(l)) => l,
        _ => return Ok(None),
    };
    let choice: usize = line.trim().parse().unwrap_or(usize::MAX);
    if choice == 0 {
        Ok(None)
    } else if (1..=targets.len()).contains(&choice) {
        Ok(Some(targets[choice - 1]))
    } else {
        println!("invalid choice — ending turn");
        Ok(None)
    }
}

fn print_hp_panel(enc: &Encounter) {
    println!("\n-- Status --");
    for c in enc.creatures() {
        let marker = if c.is_down() { "[DOWN]" } else { "[ OK ]" };
        let side = match c.side {
            Side::Pcs => "PC",
            Side::Enemies => "EN",
        };
        println!(
            "  {} {} {:<28} HP {:>3}/{:<3} AC {}",
            marker, side, c.name, c.hp, c.max_hp, c.ac
        );
    }
}
