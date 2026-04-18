//! Scene manifest — maps scene IDs to focused descriptions and typed entity
//! rosters.
//!
//! # Why this exists
//!
//! Raw adventure Markdown (LMoP chapters) is 2,000–5,000 words per file —
//! too long for Ollama's context window and useless as a structured entity
//! source.  The manifest is a hand-authored YAML that solves both problems:
//!
//! 1. **`description`** — a 50-200 word focused summary of the scene,
//!    exactly what gets sent to `/api/suggest`.
//! 2. **`entities`** — typed `Creature` records (same shape as
//!    `fixtures.yaml`) so the engine has `EntityId`s before any AI call.
//!
//! # Format
//!
//! ```yaml
//! entry: goblin_ambush
//! scenes:
//!   - id: goblin_ambush
//!     title: "Goblin Arrows"
//!     description: >
//!       A ruined wagon sits in the middle of the Triboar Trail …
//!     entities:
//!       - id: 1
//!         name: Thorn
//!         side: pcs
//!         ac: 15
//!         hp: 12
//!         max_hp: 12
//!         initiative_bonus: 3
//!         attacks:
//!           - name: Shortsword
//!             attack_bonus: 5
//!             damage_dice: "1d6+3"
//!             damage_type: piercing
//!     next: cragmaw_cave
//! ```
//!
//! Entity fields match `dm_core::entity::Creature` exactly — `serde_yaml`
//! deserialises them directly with no extra mapping layer.

use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use dm_core::entity::Creature;

#[derive(Debug, Error)]
pub enum SceneError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("scene not found: {0}")]
    NotFound(String),
    #[error("manifest has no scenes")]
    Empty,
}

/// One scene definition inside the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneDef {
    pub id: String,
    pub title: String,
    /// Focused prose sent verbatim to `/api/suggest`. Keep 50-200 words.
    pub description: String,
    pub entities: Vec<Creature>,
    /// ID of the default next scene (optional — some scenes branch).
    pub next: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ManifestFile {
    pub entry: String,
    pub scenes: Vec<SceneDef>,
}

/// Loaded scene manifest. Index is by `scene_id`.
#[derive(Debug)]
pub struct SceneManifest {
    scenes: HashMap<String, SceneDef>,
    entry: String,
}

impl SceneManifest {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, SceneError> {
        let f = File::open(path)?;
        let raw: ManifestFile = serde_yaml::from_reader(f)?;
        if raw.scenes.is_empty() {
            return Err(SceneError::Empty);
        }
        let entry = raw.entry.clone();
        let scenes = raw.scenes.into_iter().map(|s| (s.id.clone(), s)).collect();
        Ok(Self { scenes, entry })
    }

    pub fn entry_id(&self) -> &str {
        &self.entry
    }

    pub fn entry_scene(&self) -> Option<&SceneDef> {
        self.scenes.get(&self.entry)
    }

    pub fn get(&self, id: &str) -> Option<&SceneDef> {
        self.scenes.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    const SAMPLE: &str = r#"
entry: test_scene
scenes:
  - id: test_scene
    title: "Test"
    description: "Four goblins lurk in the shadows."
    entities:
      - id: 1
        name: Hero
        side: pcs
        ac: 15
        hp: 10
        max_hp: 10
        initiative_bonus: 2
        attacks:
          - name: Sword
            attack_bonus: 4
            damage_dice: "1d8+2"
            damage_type: slashing
      - id: 101
        name: Goblin
        side: enemies
        ac: 15
        hp: 7
        max_hp: 7
        initiative_bonus: 2
        attacks:
          - name: Scimitar
            attack_bonus: 4
            damage_dice: "1d6+2"
            damage_type: slashing
    next: null
"#;

    fn write_tmp(name: &str, content: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!("dm-wiki-scene-{}.yaml", name));
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        p
    }

    #[test]
    fn loads_sample_manifest() {
        let p = write_tmp("load", SAMPLE);
        let m = SceneManifest::load(&p).unwrap();
        assert_eq!(m.entry_id(), "test_scene");
        let scene = m.entry_scene().unwrap();
        assert_eq!(scene.entities.len(), 2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn rejects_empty_scenes() {
        let yaml = "entry: x\nscenes: []\n";
        let p = write_tmp("empty", yaml);
        assert!(matches!(SceneManifest::load(&p).unwrap_err(), SceneError::Empty));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let p = write_tmp("get", SAMPLE);
        let m = SceneManifest::load(&p).unwrap();
        assert!(m.get("no_such_scene").is_none());
        let _ = std::fs::remove_file(&p);
    }
}
