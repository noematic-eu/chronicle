use chronicle_engine::{Carry, FicheGot, HeirState, WorldState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const SAVE_SCHEMA: &str = "chronicle.save/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageSave {
    pub schema: String,
    pub chronicle: String,
    pub completed: Vec<String>,
    pub next_chapter: Option<String>,
    pub heir: HeirState,
    pub heritage: Vec<String>,
    pub flags: Vec<String>,
    pub monnaies: std::collections::BTreeMap<String, i32>,
    pub notes: Vec<String>,
    pub fiches: Vec<FicheGot>,
}

pub fn instance_dir(explicit: Option<PathBuf>) -> PathBuf {
    explicit.unwrap_or_else(|| PathBuf::from("instance/chronicle"))
}

pub fn save_path(instance: &Path, chronicle: &str) -> PathBuf {
    instance.join(chronicle).join("lineage.json")
}

pub fn write_lineage(instance: &Path, chronicle: &str, state: &WorldState) -> std::io::Result<()> {
    fs::create_dir_all(instance.join(chronicle))?;
    let prev = read_lineage(instance, chronicle).ok().flatten();
    let mut completed = prev
        .as_ref()
        .map(|p| p.completed.clone())
        .unwrap_or_default();
    if !completed.iter().any(|c| c == &state.chapter) {
        completed.push(state.chapter.clone());
    }
    let mut flags: Vec<String> = prev.as_ref().map(|p| p.flags.clone()).unwrap_or_default();
    for f in &state.flags {
        if !flags.contains(f) {
            flags.push(f.clone());
        }
    }
    let mut fiches = prev.as_ref().map(|p| p.fiches.clone()).unwrap_or_default();
    for f in &state.fiches {
        if !fiches.iter().any(|x| x.id == f.id) {
            fiches.push(f.clone());
        }
    }
    let save = LineageSave {
        schema: SAVE_SCHEMA.into(),
        chronicle: chronicle.into(),
        completed,
        next_chapter: state.next_chapter.clone(),
        heir: state.heir.clone(),
        heritage: state.heritage.iter().cloned().collect(),
        flags,
        monnaies: state.monnaies.clone(),
        notes: state.notes.clone(),
        fiches,
    };
    let bytes = serde_json::to_vec_pretty(&save)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(save_path(instance, chronicle), bytes)
}

pub fn read_lineage(instance: &Path, chronicle: &str) -> std::io::Result<Option<LineageSave>> {
    let p = save_path(instance, chronicle);
    if !p.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(p)?;
    let save: LineageSave = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Some(save))
}

pub fn to_carry(save: &LineageSave) -> Carry {
    Carry {
        heir: Some(save.heir.clone()),
        heritage: save.heritage.iter().cloned().collect::<BTreeSet<_>>(),
        flags: save.flags.iter().cloned().collect(),
        notes: save.notes.clone(),
        monnaies: save.monnaies.clone(),
        fiches: save.fiches.clone(),
    }
}
