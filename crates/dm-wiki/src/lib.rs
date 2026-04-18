//! dm-wiki: campaign vault I/O — fixture loading and session writing.
//!
//! Prototype 1 scope: read creature fixtures from YAML, append-only session
//! writer. No Markdown parsing, no entity linking, no lint workflow.

pub mod fixtures;
pub mod lmop;
pub mod scene;
pub mod session;
