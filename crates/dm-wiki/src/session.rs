//! Append-only session log writer.
//!
//! Creates `<campaign>/sessions/sessionNN.md` with YAML frontmatter matching
//! the schema in `CLAUDE.md`, then exposes `append` for combat log lines. No
//! read-back — a replay layer would parse the file separately.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::Local;
use thiserror::Error;

use dm_core::effect::Effect;
use dm_core::entity::Creature;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
}

pub struct SessionWriter {
    path: PathBuf,
}

impl SessionWriter {
    /// Create or truncate `<campaign_dir>/sessions/sessionNN.md` and write the
    /// frontmatter + opening headings. PC names go into the `players:` field.
    pub fn open_or_create(
        campaign_dir: &Path,
        session_number: u32,
        pcs: &[Creature],
    ) -> Result<Self, SessionError> {
        let sessions_dir = campaign_dir.join("sessions");
        fs::create_dir_all(&sessions_dir)?;
        let filename = format!("session{:02}.md", session_number);
        let path = sessions_dir.join(filename);

        let date = Local::now().format("%Y-%m-%d").to_string();
        let players = pcs
            .iter()
            .map(|c| format!("\"[[{}]]\"", c.name))
            .collect::<Vec<_>>()
            .join(", ");

        let header = format!(
            "---\n\
             type: session\n\
             number: {n}\n\
             real_date: {date}\n\
             in_world_date: unknown\n\
             players: [{players}]\n\
             tags: [session, prototype]\n\
             ---\n\
             \n\
             # Session {n:02} — Prototype 1\n\
             \n\
             ## Combat Log\n\
             \n",
            n = session_number,
            date = date,
            players = players
        );

        let mut f = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;
        f.write_all(header.as_bytes())?;

        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn append(&mut self, line: &str) -> Result<(), SessionError> {
        let mut f = OpenOptions::new().append(true).open(&self.path)?;
        writeln!(f, "{}", line)?;
        Ok(())
    }

    /// Write an Effect as a single bullet line.
    pub fn log_effect(&mut self, effect: &Effect) -> Result<(), SessionError> {
        let line = format!("- {}", effect.narrative());
        self.append(&line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dm_core::entity::{Attack, DamageType, EntityId, Side};

    fn tmp_dir(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("dm-wiki-session-{}", name));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn hero() -> Creature {
        Creature {
            id: EntityId(1),
            name: "Hero".into(),
            side: Side::Pcs,
            ac: 15,
            hp: 20,
            max_hp: 20,
            initiative_bonus: 2,
            attacks: vec![Attack {
                name: "Sword".into(),
                attack_bonus: 5,
                damage_dice: "1d8+3".into(),
                damage_type: DamageType::Slashing,
            }],
        }
    }

    #[test]
    fn creates_file_with_frontmatter() {
        let dir = tmp_dir("creates");
        let pcs = vec![hero()];
        let w = SessionWriter::open_or_create(&dir, 1, &pcs).unwrap();
        let contents = fs::read_to_string(w.path()).unwrap();
        assert!(contents.contains("type: session"));
        assert!(contents.contains("number: 1"));
        assert!(contents.contains("\"[[Hero]]\""));
        assert!(contents.contains("## Combat Log"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn appends_lines() {
        let dir = tmp_dir("appends");
        let pcs = vec![hero()];
        let mut w = SessionWriter::open_or_create(&dir, 2, &pcs).unwrap();
        w.append("- first line").unwrap();
        w.append("- second line").unwrap();
        let contents = fs::read_to_string(w.path()).unwrap();
        assert!(contents.contains("- first line"));
        assert!(contents.contains("- second line"));
        fs::remove_dir_all(&dir).unwrap();
    }
}
