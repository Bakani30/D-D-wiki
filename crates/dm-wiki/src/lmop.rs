//! LMoP Markdown parser — extracts scene descriptions and monster stat blocks
//! from the translated Thai adventure files.
//!
//! # Story file structure
//! ```text
//! --- (frontmatter) ---
//! ## Section heading      ← H2 = scene boundary
//!   > blockquote          ← first blockquote per H2 = read-aloud description
//!   ### Sub-heading        ← H3 inside a scene (NOT a new scene)
//! ```
//!
//! # Monster appendix structure
//! ```text
//! ### ThaiName (EnglishName)   ← H3 intro (ignored)
//! ## ThaiName (EnglishName)    ← H2 = stat block start
//!   **ระดับการป้องกัน (Armor Class: AC)** 15 (armor)
//!   **ฮิตพอยต์** 7 (2d6)
//!   | STR | DEX | CON | ...   ← ability score table header
//!   | 8 (-1) | 14 (+2) | ...  ← ability score data row
//!   ### แอ็คชัน (Actions)
//!   ***WeaponThai (English)** type:* +4 ให้การทอยโจมตี ... *โดน:* 5 (1d6+2) ... (slashing damage)
//! ```

use dm_core::entity::{Attack, Creature, DamageType, EntityId, Side};

// ── Public output types ───────────────────────────────────────────────────────

/// A scene extracted from a chapter markdown file.
#[derive(Debug, Clone)]
pub struct ExtractedScene {
    /// Auto-generated slug: `scene_01`, `scene_02`, …
    pub id: String,
    /// Thai heading text from the `## ` line.
    pub title: String,
    /// First `> blockquote` in the section — the DM read-aloud text.
    pub description: String,
}

/// A parsed attack action from the `### แอ็คชัน (Actions)` section.
#[derive(Debug, Clone)]
pub struct ExtractedAttack {
    /// English name from parentheses: `(Scimitar)` → `"Scimitar"`.
    pub name: String,
    pub attack_bonus: i32,
    /// Normalised dice expression: `"1d6+2"`, `"2d8+2"`.
    pub damage_dice: String,
    /// Lowercase English type word: `"slashing"`, `"piercing"`, …
    pub damage_type: String,
}

/// A monster stat block extracted from the appendix markdown.
#[derive(Debug, Clone)]
pub struct ExtractedMonster {
    pub name_thai: String,
    pub name_english: String,
    pub ac: i32,
    pub hp: i32,
    /// DEX modifier, used as the initiative bonus.
    pub initiative_bonus: i32,
    pub attacks: Vec<ExtractedAttack>,
}

impl ExtractedMonster {
    /// Convert to a `Creature` with the given numeric ID and side.
    pub fn to_creature(&self, id: u32, number: u32, side: Side) -> Creature {
        let name = if number > 0 {
            format!("{} {} ({})", self.name_thai, number, self.name_english)
        } else {
            format!("{} ({})", self.name_thai, self.name_english)
        };
        Creature {
            id: EntityId(id),
            name,
            side,
            ac: self.ac,
            hp: self.hp,
            max_hp: self.hp,
            initiative_bonus: self.initiative_bonus,
            attacks: self
                .attacks
                .iter()
                .map(|a| Attack {
                    name: a.name.clone(),
                    attack_bonus: a.attack_bonus,
                    damage_dice: a.damage_dice.clone(),
                    damage_type: parse_damage_type(&a.damage_type),
                })
                .collect(),
        }
    }
}

// ── Scene extraction ──────────────────────────────────────────────────────────

/// Extract all H2-level scenes from an LMoP chapter file.
///
/// Each `## Heading` starts a new scene. The description is the content of the
/// first `>` blockquote under that heading — the DM read-aloud box.
pub fn extract_scenes(md: &str) -> Vec<ExtractedScene> {
    let lines: Vec<&str> = md.lines().collect();
    let body_start = skip_frontmatter(&lines);

    let mut scenes: Vec<ExtractedScene> = Vec::new();

    // Per-scene state
    let mut current_title: Option<String> = None;
    let mut bq_lines: Vec<String> = Vec::new();
    let mut in_bq = false;
    let mut bq_captured = false;

    for line in &lines[body_start..] {
        if line.starts_with("## ") {
            // Finalize previous scene
            if let Some(ref title) = current_title {
                scenes.push(make_scene(scenes.len(), title, &bq_lines));
            }
            current_title = Some(line[3..].trim().to_string());
            bq_lines.clear();
            in_bq = false;
            bq_captured = false;
        } else if current_title.is_some() && !bq_captured {
            if line.starts_with("> ") {
                in_bq = true;
                bq_lines.push(line[2..].to_string());
            } else if *line == ">" {
                if in_bq {
                    bq_lines.push(String::new());
                }
            } else if in_bq {
                // Non-blockquote line after a blockquote → description done.
                in_bq = false;
                bq_captured = true;
            }
        }
    }
    // Finalize last scene
    if let Some(ref title) = current_title {
        scenes.push(make_scene(scenes.len(), title, &bq_lines));
    }

    scenes
}

// ── Monster extraction ────────────────────────────────────────────────────────

/// Extract all monster stat blocks from the appendix markdown.
///
/// Monster entries start with an H2 of the form `## ThaiName (EnglishName)`.
/// Section headings without `(EnglishName)` parens (like `## ข้อมูลสถิติ...`)
/// are skipped.
pub fn extract_monsters(md: &str) -> Vec<ExtractedMonster> {
    let lines: Vec<&str> = md.lines().collect();
    let body_start = skip_frontmatter(&lines);

    let mut monsters: Vec<ExtractedMonster> = Vec::new();

    // Mutable builder state
    let mut name_thai = String::new();
    let mut name_english = String::new();
    let mut ac: Option<i32> = None;
    let mut hp: Option<i32> = None;
    let mut initiative_bonus: Option<i32> = None;
    let mut attacks: Vec<ExtractedAttack> = Vec::new();
    let mut in_actions = false;
    let mut ability_header_seen = false;
    let mut building = false;

    for line in &lines[body_start..] {
        if line.starts_with("## ") {
            // Finalize previous monster
            if let Some(m) = flush_monster(
                building,
                &name_thai,
                &name_english,
                ac,
                hp,
                initiative_bonus,
                &attacks,
            ) {
                monsters.push(m);
            }
            // Start new — only if heading has (EnglishName) parens
            let heading = line[3..].trim();
            if let Some((thai, english)) = parse_bilingual_heading(heading) {
                name_thai = thai;
                name_english = english;
                ac = None;
                hp = None;
                initiative_bonus = None;
                attacks.clear();
                in_actions = false;
                ability_header_seen = false;
                building = true;
            } else {
                building = false;
            }
        } else if building {
            if line.contains("**ระดับการป้องกัน") || line.contains("Armor Class: AC") {
                ac = extract_first_int_after_closing_bold(line);
            } else if line.starts_with("**ฮิตพอยต์") {
                hp = extract_first_int_after_closing_bold(line);
            } else if line.contains("| STR |") {
                ability_header_seen = true;
            } else if ability_header_seen
                && line.starts_with("| ")
                && !line.starts_with("| ---")
                && !line.contains("STR")
            {
                initiative_bonus = parse_dex_modifier(line);
                ability_header_seen = false;
            } else if line.starts_with("### แอ็คชัน") || line.contains("(Actions)") {
                in_actions = true;
            } else if line.starts_with("### ") && !line.contains("แอ็คชัน") {
                // Another H3 section — exit actions block
                in_actions = false;
            } else if in_actions && line.starts_with("***") && line.contains("ให้การทอยโจมตี") {
                if let Some(atk) = parse_attack_line(line) {
                    attacks.push(atk);
                }
            }
        }
    }
    // Finalize last monster
    if let Some(m) = flush_monster(
        building,
        &name_thai,
        &name_english,
        ac,
        hp,
        initiative_bonus,
        &attacks,
    ) {
        monsters.push(m);
    }

    monsters
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn skip_frontmatter(lines: &[&str]) -> usize {
    if lines.first().copied() != Some("---") {
        return 0;
    }
    lines
        .iter()
        .skip(1)
        .position(|l| *l == "---")
        .map(|p| p + 2)
        .unwrap_or(0)
}

/// Split `"กอบลิน (Goblin)"` into `("กอบลิน", "Goblin")`.
/// Returns `None` if there are no ASCII-only parentheses.
fn parse_bilingual_heading(h: &str) -> Option<(String, String)> {
    let open = h.rfind('(')?;
    let close = h[open..].find(')')? + open;
    let english = h[open + 1..close].trim();
    // Reject if the English part is Thai / non-ASCII (e.g. size table headers)
    if english.is_empty() || !english.is_ascii() {
        return None;
    }
    let thai = h[..open].trim().to_string();
    if thai.is_empty() {
        return None;
    }
    Some((thai, english.to_string()))
}

fn make_scene(index: usize, title: &str, bq: &[String]) -> ExtractedScene {
    let desc = bq
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    ExtractedScene {
        id: format!("scene_{:02}", index + 1),
        title: title.to_string(),
        description: if desc.is_empty() { "TODO: add description".to_string() } else { desc },
    }
}

fn flush_monster(
    building: bool,
    name_thai: &str,
    name_english: &str,
    ac: Option<i32>,
    hp: Option<i32>,
    init: Option<i32>,
    atks: &[ExtractedAttack],
) -> Option<ExtractedMonster> {
    if !building || name_english.is_empty() {
        return None;
    }
    Some(ExtractedMonster {
        name_thai: name_thai.to_string(),
        name_english: name_english.to_string(),
        ac: ac.unwrap_or(10),
        hp: hp.unwrap_or(1),
        initiative_bonus: init.unwrap_or(0),
        attacks: atks.to_vec(),
    })
}

/// Parse the integer immediately after the last closing `**` on the line.
///
/// `"**ระดับการป้องกัน (Armor Class: AC)** 15 (เกราะหนัง...)"` → `Some(15)`
fn extract_first_int_after_closing_bold(line: &str) -> Option<i32> {
    // Find the last occurrence of "**" — the closing bold marker.
    let close = line.rfind("**")?;
    let rest = line[close + 2..].trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    rest[..end].parse().ok()
}

/// Parse the DEX modifier (2nd data column) from an ability score table row.
///
/// `"| 8 (-1) | 14 (+2) | 10 (+0) | ..."` → `Some(2)`
fn parse_dex_modifier(line: &str) -> Option<i32> {
    // Split on `|`; parts[1] = STR, parts[2] = DEX
    let parts: Vec<&str> = line.split('|').collect();
    let dex_cell = parts.get(2)?;
    let open = dex_cell.find('(')?;
    let close = dex_cell[open..].find(')')? + open;
    let modifier_str = dex_cell[open + 1..close].trim();
    modifier_str.parse().ok()
}

/// Parse a single attack line from the `### แอ็คชัน` section.
///
/// Handles two observed formats:
/// - `***Weapon (English)** type:* +N ...` (mixed bold+italic)
/// - `***Weapon (English)*** *type:*       +N ...` (separate)
///
/// Both have `ให้การทอยโจมตี` after the attack bonus and `*โดน:*` before damage.
fn parse_attack_line(line: &str) -> Option<ExtractedAttack> {
    // English weapon name — between last `(` and `)` in the bold section,
    // i.e. before the first space after the triple-asterisk region.
    let after_stars = line.strip_prefix("***")?;

    // Find the English name inside the first set of parens in the weapon label.
    // The weapon label ends at `**` (for format 1) or `***` (for format 2).
    let label_end = after_stars
        .find("** ")
        .or_else(|| after_stars.find("***"))
        .unwrap_or(after_stars.len());
    let label = &after_stars[..label_end];

    let name = {
        let open = label.rfind('(')?;
        let close = label[open..].find(')')? + open;
        let n = label[open + 1..close].trim();
        if n.is_empty() || !n.is_ascii() {
            return None;
        }
        n.to_string()
    };

    // Attack bonus: "+N ให้การทอยโจมตี"
    let atk_marker_pos = line.find("ให้การทอยโจมตี")?;
    let before_marker = &line[..atk_marker_pos];
    let plus_pos = before_marker.rfind('+')?;
    let num_str = before_marker[plus_pos + 1..].trim_end();
    let end = num_str
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(num_str.len());
    if end == 0 {
        return None;
    }
    let attack_bonus: i32 = num_str[..end].parse().ok()?;

    // Damage dice and type: "*โดน:* N (XdY+Z) ... (type damage)"
    let hit_pos = line.find("*โดน:*").or_else(|| line.find("*Hit:*"))?;
    let after_hit = &line[hit_pos..];
    // Find the closing "* " of the italic marker (e.g. "*โดน:* ")
    let star_space = after_hit.find("* ").map(|p| p + 2)?;
    let damage_section = &after_hit[star_space..];

    // Dice expression in the first parens: "(1d6 + 2)"
    let dice_open = damage_section.find('(')?;
    let dice_close = damage_section[dice_open..].find(')')? + dice_open;
    let dice_raw = damage_section[dice_open + 1..dice_close].trim();
    let damage_dice = dice_raw
        .replace(" + ", "+")
        .replace(" - ", "-")
        .replace(" +", "+")
        .replace("+ ", "+");

    // Damage type: last "(word damage)" at end of line
    let lower = line.to_lowercase();
    let suffix = " damage)";
    let suffix_pos = lower.rfind(suffix)?;
    let type_open = lower[..suffix_pos].rfind('(')?;
    let damage_type = line[type_open + 1..suffix_pos].trim().to_lowercase();

    Some(ExtractedAttack {
        name,
        attack_bonus,
        damage_dice,
        damage_type,
    })
}

fn parse_damage_type(s: &str) -> DamageType {
    match s.to_lowercase().trim() {
        "slashing" => DamageType::Slashing,
        "piercing" => DamageType::Piercing,
        "bludgeoning" => DamageType::Bludgeoning,
        "fire" => DamageType::Fire,
        "cold" => DamageType::Cold,
        "poison" => DamageType::Poison,
        "acid" => DamageType::Acid,
        "lightning" => DamageType::Lightning,
        "thunder" => DamageType::Thunder,
        "psychic" => DamageType::Psychic,
        "necrotic" => DamageType::Necrotic,
        "radiant" => DamageType::Radiant,
        "force" => DamageType::Force,
        _ => DamageType::Bludgeoning,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──

    #[test]
    fn bilingual_heading_goblin() {
        let (thai, english) = parse_bilingual_heading("กอบลิน (Goblin)").unwrap();
        assert_eq!(thai, "กอบลิน");
        assert_eq!(english, "Goblin");
    }

    #[test]
    fn bilingual_heading_no_english_parens() {
        // Thai-only heading should return None
        assert!(parse_bilingual_heading("ข้อมูลสถิติของมอนสเตอร์").is_none());
    }

    #[test]
    fn bilingual_heading_thai_parens_rejected() {
        // Heading with Thai inside parens should return None (non-ASCII)
        assert!(parse_bilingual_heading("ขนาด (กลาง)").is_none());
    }

    #[test]
    fn ac_extraction() {
        let line = "**ระดับการป้องกัน (Armor Class: AC)** 15 (เกราะหนัง, โล่ )";
        assert_eq!(extract_first_int_after_closing_bold(line), Some(15));
    }

    #[test]
    fn hp_extraction() {
        let line = "**ฮิตพอยต์** 7 (2d6)";
        assert_eq!(extract_first_int_after_closing_bold(line), Some(7));
    }

    #[test]
    fn dex_modifier_positive() {
        let line = "| 8 (-1) | 14 (+2) | 10 (+0) | 10 (+0) | 8 (-1) | 8 (-1) |";
        assert_eq!(parse_dex_modifier(line), Some(2));
    }

    #[test]
    fn dex_modifier_negative() {
        let line = "| 15 (+2) | 8 (-1) | 13 (+1) | 8 (-1) | 11 (+0) | 9 (-1) |";
        assert_eq!(parse_dex_modifier(line), Some(-1));
    }

    // ── Attack parsing ──

    const SCIMITAR_LINE: &str = "***ดาบโค้ง (Scimitar)** การโจมตีด้วยอาวุธระยะประชิด:* +4 ให้การทอยโจมตี, ระยะ 5 ฟุต, เป้าหมายเดียว *โดน:* 5 (1d6 + 2) ความเสียหายแบบเฉือน (slashing damage)";
    const MORNINGSTAR_LINE: &str = "***ลูกตุ้มหนาม (Morningstar)*** *การโจมตีด้วยอาวุธระยะประชิด:* +4 ให้การทอยโจมตี, ระยะ 5 ฟุต, เป้าหมายเดียว. *โดน:* 11 (2d8 + 2) ความเสียหายแบบแทง (Piercing damage)";
    const SHORTBOW_LINE: &str = "***ธนูสั้น (Shortbow)** การโจมตีด้วยอาวุธระยะไกล:* +4 ให้การทอยโจมตี, ระยะไกล 80 ฟุต/320 ฟุต, เป้าหมายเดียว *โดน:* 5 (1d6 + 2) ความเสียหายแบบแทง (Piercing damage)";

    #[test]
    fn parse_scimitar() {
        let atk = parse_attack_line(SCIMITAR_LINE).unwrap();
        assert_eq!(atk.name, "Scimitar");
        assert_eq!(atk.attack_bonus, 4);
        assert_eq!(atk.damage_dice, "1d6+2");
        assert_eq!(atk.damage_type, "slashing");
    }

    #[test]
    fn parse_morningstar() {
        let atk = parse_attack_line(MORNINGSTAR_LINE).unwrap();
        assert_eq!(atk.name, "Morningstar");
        assert_eq!(atk.attack_bonus, 4);
        assert_eq!(atk.damage_dice, "2d8+2");
        assert_eq!(atk.damage_type, "piercing");
    }

    #[test]
    fn parse_shortbow() {
        let atk = parse_attack_line(SHORTBOW_LINE).unwrap();
        assert_eq!(atk.name, "Shortbow");
        assert_eq!(atk.attack_bonus, 4);
        assert_eq!(atk.damage_dice, "1d6+2");
        assert_eq!(atk.damage_type, "piercing");
    }

    // ── Integration: full goblin stat block ──

    const GOBLIN_BLOCK: &str = r#"---
title: test
---
### กอบลิน (Goblin)

ก็อบลินเป็นพวกจิตใจชั่วร้าย

## กอบลิน (Goblin)

*ขนาดเล็ก รูปร่างแบบมนุษย์ (กอบลินนอยด์) (), เป็นกลาง ชั่วร้าย*

**ระดับการป้องกัน (Armor Class: AC)** 15 (เกราะหนัง, โล่ )

**ฮิตพอยต์** 7 (2d6)

**ความเร็ว** 30 ฟุต

| STR | DEX | CON | INT | WIS | CHA |
| --- | --- | --- | --- | --- | --- |
| 8 (-1) | 14 (+2) | 10 (+0) | 10 (+0) | 8 (-1) | 8 (-1) |

**ทักษะ** การลอบเร้น +6

### คุณลักษณะพิเศษ (Traits)

***หลบหนีอย่างรวดเร็ว (Nimble Escape)*** กอบลินสามารถใช้ การผละหนี

### แอ็คชัน (Actions)

***ดาบโค้ง (Scimitar)** การโจมตีด้วยอาวุธระยะประชิด:* +4 ให้การทอยโจมตี, ระยะ 5 ฟุต, เป้าหมายเดียว *โดน:* 5 (1d6 + 2) ความเสียหายแบบเฉือน (slashing damage)

***ธนูสั้น (Shortbow)** การโจมตีด้วยอาวุธระยะไกล:* +4 ให้การทอยโจมตี, ระยะไกล 80 ฟุต/320 ฟุต, เป้าหมายเดียว *โดน:* 5 (1d6 + 2) ความเสียหายแบบแทง (Piercing damage)
"#;

    #[test]
    fn goblin_full_stat_block() {
        let monsters = extract_monsters(GOBLIN_BLOCK);
        assert_eq!(monsters.len(), 1);
        let g = &monsters[0];
        assert_eq!(g.name_english, "Goblin");
        assert_eq!(g.ac, 15);
        assert_eq!(g.hp, 7);
        assert_eq!(g.initiative_bonus, 2);
        assert_eq!(g.attacks.len(), 2);
        assert_eq!(g.attacks[0].name, "Scimitar");
        assert_eq!(g.attacks[0].attack_bonus, 4);
        assert_eq!(g.attacks[0].damage_dice, "1d6+2");
        assert_eq!(g.attacks[0].damage_type, "slashing");
        assert_eq!(g.attacks[1].name, "Shortbow");
        assert_eq!(g.attacks[1].damage_type, "piercing");
    }

    #[test]
    fn goblin_to_creature_roundtrip() {
        let monsters = extract_monsters(GOBLIN_BLOCK);
        let creature = monsters[0].to_creature(101, 1, Side::Enemies);
        assert_eq!(creature.id, EntityId(101));
        assert_eq!(creature.ac, 15);
        assert_eq!(creature.hp, 7);
        // Verify damage_type roundtrip through parse_damage_type
        assert!(matches!(creature.attacks[0].damage_type, DamageType::Slashing));
        assert!(matches!(creature.attacks[1].damage_type, DamageType::Piercing));
    }

    // ── Scene extraction ──

    const SCENE_MD: &str = r#"---
title: "chapter one"
---
## บทที่ 1: ลูกศรของก๊อบลิน

narrative text here

> ในเมืองเนเวอร์วินเทอร์ ดวอร์ฟนามว่า กันเดรน ได้ขอให้พวกคุณนำเกวียน

### sub-section

more text

## ก๊อบลินซุ่มโจมตี

> คุณอยู่บนทางเกวียนไตรบอร์มาราวครึ่งวัน เมื่อคุณถึงทางโค้ง คุณก็สังเกตเห็นม้าตายสองตัว

DM instructions here
"#;

    #[test]
    fn scene_extraction_basic() {
        let scenes = extract_scenes(SCENE_MD);
        assert_eq!(scenes.len(), 2);
        assert_eq!(scenes[0].id, "scene_01");
        assert!(scenes[0].title.contains("ลูกศรของก๊อบลิน"));
        assert!(scenes[0].description.contains("กันเดรน"));
        assert_eq!(scenes[1].id, "scene_02");
        assert!(scenes[1].title.contains("ก๊อบลินซุ่มโจมตี"));
        assert!(scenes[1].description.contains("ทางเกวียน"));
    }

    #[test]
    fn scene_without_blockquote_gets_todo() {
        let md = "## Scene No Blockquote\n\nJust some text without a quote.\n";
        let scenes = extract_scenes(md);
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].description, "TODO: add description");
    }
}
