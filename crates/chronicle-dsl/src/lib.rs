//! Author Markdown (YAML fence) → Chronicle IR.

use chronicle_engine::{ChapterIr, ChronicleMeta, Ir, SCHEMA};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DslError {
    #[error("{0}")]
    Message(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, DslError>;

/// Packs live in a sibling `chronicle-stories` repo (or `CHRONICLE_STORIES`).
pub fn stories_dir() -> PathBuf {
    std::env::var("CHRONICLE_STORIES")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../chronicle-stories")
        })
}

#[derive(Debug, Deserialize)]
struct FileSpec {
    #[serde(default)]
    chronicle: Option<ChronicleMeta>,
    #[serde(flatten)]
    chapter: ChapterIr,
}

#[derive(Debug, Clone)]
pub struct Campaign {
    pub meta: ChronicleMeta,
    pub entry: String,
    pub chapters: BTreeMap<String, Ir>,
}

impl Campaign {
    pub fn get(&self, id: &str) -> Option<&Ir> {
        self.chapters.get(id)
    }

    pub fn order(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut id = Some(self.entry.clone());
        while let Some(cur) = id {
            if out.contains(&cur) {
                break;
            }
            let next = self
                .chapters
                .get(&cur)
                .and_then(|ir| ir.chapter.next.clone());
            out.push(cur);
            id = next;
        }
        out
    }
}

#[derive(Debug, Deserialize)]
struct CampagneFile {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    lieu: String,
    #[serde(default)]
    entry: String,
}

pub fn compile_path(path: &Path) -> Result<Ir> {
    if path.is_dir() {
        let camp = compile_campaign(path)?;
        camp.get(&camp.entry)
            .cloned()
            .ok_or_else(|| DslError::Message(format!("entry {} missing", camp.entry)))
    } else {
        compile_file(path)
    }
}

pub fn compile_campaign(dir: &Path) -> Result<Campaign> {
    let mut files = Vec::new();
    collect_md(dir, &mut files)?;
    files.sort();
    let mut chapters = BTreeMap::new();
    let mut meta = ChronicleMeta {
        id: String::new(),
        title: String::new(),
        lieu: String::new(),
    };
    for f in &files {
        match compile_file(f) {
            Ok(ir) => {
                if meta.id.is_empty() {
                    meta = ir.chronicle.clone();
                }
                chapters.insert(ir.chapter.id.clone(), ir);
            }
            Err(DslError::Message(m)) if m.contains("no yaml fence") => continue,
            Err(e) => return Err(e),
        }
    }
    if chapters.is_empty() {
        return Err(DslError::Message(format!(
            "no playable chapters under {}",
            dir.display()
        )));
    }
    let mut entry = String::new();
    for name in ["campagne.yaml", "campagne.yml", "campaign.yaml"] {
        let p = dir.join(name);
        if p.exists() {
            let raw = std::fs::read_to_string(&p)?;
            let spec: CampagneFile = serde_yaml::from_str(&raw)?;
            if !spec.id.is_empty() {
                meta.id = spec.id;
            }
            if !spec.title.is_empty() {
                meta.title = spec.title;
            }
            if !spec.lieu.is_empty() {
                meta.lieu = spec.lieu;
            }
            entry = spec.entry;
            break;
        }
    }
    if entry.is_empty() {
        entry = infer_entry(&chapters);
    }
    if !chapters.contains_key(&entry) {
        return Err(DslError::Message(format!(
            "entry `{entry}` not in campaign"
        )));
    }
    for ir in chapters.values_mut() {
        ir.chronicle = meta.clone();
    }
    Ok(Campaign {
        meta,
        entry,
        chapters,
    })
}

fn infer_entry(chapters: &BTreeMap<String, Ir>) -> String {
    let targets: std::collections::BTreeSet<String> = chapters
        .values()
        .filter_map(|ir| ir.chapter.next.clone())
        .collect();
    chapters
        .keys()
        .find(|id| !targets.contains(*id))
        .cloned()
        .or_else(|| chapters.keys().next().cloned())
        .unwrap_or_default()
}

pub fn compile_file(path: &Path) -> Result<Ir> {
    let src = std::fs::read_to_string(path)?;
    compile_markdown(&src, &path.display().to_string())
}

pub fn compile_markdown(src: &str, file: &str) -> Result<Ir> {
    let yaml =
        extract_yaml(src).ok_or_else(|| DslError::Message(format!("{file}: no yaml fence")))?;
    let spec: FileSpec = serde_yaml::from_str(&yaml)?;
    if spec.chapter.petitions.is_empty() {
        return Err(DslError::Message(format!("{file}: petitions is empty")));
    }
    if spec.chapter.set_piece.choices.is_empty() {
        return Err(DslError::Message(format!(
            "{file}: set_piece has no choices"
        )));
    }
    for p in &spec.chapter.petitions {
        if p.choices.is_empty() {
            return Err(DslError::Message(format!(
                "{file}: petition {} has no choices",
                p.id
            )));
        }
    }
    let chronicle = spec.chronicle.unwrap_or(ChronicleMeta {
        id: spec.chapter.partie.clone(),
        title: spec.chapter.title.clone(),
        lieu: String::new(),
    });
    let ir = Ir {
        schema: SCHEMA.into(),
        chronicle,
        chapter: spec.chapter,
    };
    ir.validate()
        .map_err(|e| DslError::Message(format!("{file}: {e}")))?;
    Ok(ir)
}

fn extract_yaml(src: &str) -> Option<String> {
    let trimmed = src.trim_start();
    if !trimmed.starts_with('#') && !trimmed.starts_with("```") && looks_like_yaml(trimmed) {
        return Some(trimmed.to_string());
    }
    let mut lines = src.lines();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t == "```yaml" || t == "```yml" {
            let mut buf = String::new();
            for body in lines.by_ref() {
                if body.trim() == "```" {
                    return Some(buf);
                }
                buf.push_str(body);
                buf.push('\n');
            }
            return None;
        }
    }
    None
}

fn looks_like_yaml(src: &str) -> bool {
    src.starts_with("id:") || src.starts_with("chronicle:") || src.starts_with("title:")
}

fn collect_md(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<()> {
    let rd = std::fs::read_dir(dir)?;
    for ent in rd.flatten() {
        let p = ent.path();
        if p.is_dir() {
            collect_md(&p, out)?;
        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
            out.push(p);
        }
    }
    Ok(())
}

pub fn to_json(ir: &Ir) -> Result<String> {
    serde_json::to_string_pretty(ir).map_err(|e| DslError::Message(e.to_string()))
}
