use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const NOTES_CAP: usize = 26;

#[derive(Debug, Clone, Default)]
pub struct Carry {
    pub heir: Option<HeirState>,
    pub heritage: BTreeSet<String>,
    pub flags: BTreeSet<String>,
    pub notes: Vec<String>,
    pub monnaies: BTreeMap<String, i32>,
    pub fiches: Vec<FicheGot>,
}

impl Carry {
    pub fn from_state(state: &WorldState) -> Self {
        Self {
            heir: Some(state.heir.clone()),
            heritage: state.heritage.clone(),
            flags: state.flags.clone(),
            notes: state.notes.clone(),
            monnaies: state.monnaies.clone(),
            fiches: state.fiches.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeirState {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FicheGot {
    pub id: String,
    pub nom: String,
    pub texte: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldState {
    pub chapter: String,
    pub mode: String,
    pub petition_i: usize,
    pub cursor: usize,
    pub monnaies: BTreeMap<String, i32>,
    pub flags: BTreeSet<String>,
    pub notes: Vec<String>,
    pub heritage: BTreeSet<String>,
    pub fiches: Vec<FicheGot>,
    pub heir: HeirState,
    pub narrative: String,
    pub last_choice: Option<String>,
    #[serde(default)]
    pub next_chapter: Option<String>,
    #[serde(default)]
    pub overlay: Option<String>,
}

impl WorldState {
    pub fn play_mode(&self) -> &str {
        self.overlay.as_deref().unwrap_or(self.mode.as_str())
    }
}
