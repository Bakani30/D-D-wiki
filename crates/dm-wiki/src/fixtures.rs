//! YAML fixture loader for encounters.
//!
//! Accepts a top-level sequence of `Creature` records — each carrying its own
//! `side` field, so PCs and enemies live in one list. Example:
//!
//! ```yaml
//! - id: 1
//!   name: Shalefire Stoutheart
//!   side: pcs
//!   ac: 16
//!   hp: 13
//!   max_hp: 13
//!   initiative_bonus: 1
//!   attacks:
//!     - name: Handaxe
//!       attack_bonus: 6
//!       damage_dice: "1d6+4"
//!       damage_type: slashing
//! ```

use std::fs::File;
use std::io;
use std::path::Path;

use thiserror::Error;

use dm_core::entity::Creature;

#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("io error reading fixture: {0}")]
    Io(#[from] io::Error),
    #[error("yaml parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("fixture is empty")]
    Empty,
}

pub fn load_creatures<P: AsRef<Path>>(path: P) -> Result<Vec<Creature>, FixtureError> {
    let f = File::open(path)?;
    let creatures: Vec<Creature> = serde_yaml::from_reader(f)?;
    if creatures.is_empty() {
        return Err(FixtureError::Empty);
    }
    Ok(creatures)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dm_core::entity::Side;
    use std::io::Write;

    #[test]
    fn loads_minimal_yaml() {
        let yaml = r#"
- id: 1
  name: Hero
  side: pcs
  ac: 15
  hp: 20
  max_hp: 20
  initiative_bonus: 2
  attacks:
    - name: Sword
      attack_bonus: 5
      damage_dice: "1d8+3"
      damage_type: slashing
- id: 2
  name: Kobold
  side: enemies
  ac: 12
  hp: 5
  max_hp: 5
  initiative_bonus: 1
  attacks:
    - name: Dagger
      attack_bonus: 3
      damage_dice: "1d4+1"
      damage_type: piercing
"#;
        let tmp = std::env::temp_dir().join("dm-wiki-fixture-test.yaml");
        {
            let mut f = File::create(&tmp).unwrap();
            f.write_all(yaml.as_bytes()).unwrap();
        }
        let cs = load_creatures(&tmp).unwrap();
        assert_eq!(cs.len(), 2);
        assert_eq!(cs[0].side, Side::Pcs);
        assert_eq!(cs[1].side, Side::Enemies);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn rejects_empty_yaml() {
        let tmp = std::env::temp_dir().join("dm-wiki-fixture-empty.yaml");
        {
            let mut f = File::create(&tmp).unwrap();
            f.write_all(b"[]").unwrap();
        }
        let err = load_creatures(&tmp).unwrap_err();
        assert!(matches!(err, FixtureError::Empty));
        let _ = std::fs::remove_file(&tmp);
    }
}
