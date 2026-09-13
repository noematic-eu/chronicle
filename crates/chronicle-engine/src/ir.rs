use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const SCHEMA: &str = "chronicle.ir/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ir {
    pub schema: String,
    pub chronicle: ChronicleMeta,
    pub chapter: ChapterIr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronicleMeta {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub lieu: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterIr {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub partie: String,
    #[serde(default)]
    pub saison: String,
    #[serde(default)]
    pub dates: String,
    #[serde(default)]
    pub lieux: Vec<String>,
    #[serde(default)]
    pub sanctuaire: String,
    #[serde(default)]
    pub gazette: String,
    pub monnaies: Vec<CurrencyIr>,
    pub heir: HeirIr,
    pub briefing: TextBlock,
    pub petitions: Vec<PetitionIr>,
    pub set_piece: EncounterIr,
    pub debrief: DebriefIr,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub heritage_default: Vec<String>,
    /// `carry` (default): keep the heir from the previous chapter.
    /// `chapter`: this chapter's `heir` block wins (time skip).
    #[serde(default = "carry_policy")]
    pub heir_policy: String,
    /// If non-empty, after the briefing the player picks one place;
    /// only that place's petitions run. History is which doors exist.
    #[serde(default)]
    pub lieux_jouables: Vec<LieuJouable>,
    /// If true, a choice that lists `heritage` also adds `cheval` when
    /// the `chevaux` currency is > 0. France 1.1 only; not a global rule.
    #[serde(default)]
    pub cheval_si_chevaux: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LieuJouable {
    pub id: String,
    pub nom: String,
    #[serde(default)]
    pub texte: String,
    #[serde(default)]
    pub petitions: Vec<String>,
    #[serde(default)]
    pub when_flag: Option<String>,
    #[serde(default)]
    pub when_not_flag: Option<String>,
}

fn carry_policy() -> String {
    "carry".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyIr {
    pub id: String,
    pub label: String,
    pub start: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeirIr {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBlock {
    #[serde(default)]
    pub masthead: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetitionIr {
    pub id: String,
    #[serde(default)]
    pub title: String,
    pub text: String,
    pub choices: Vec<ChoiceIr>,
    #[serde(default)]
    pub when_flag: Option<String>,
    #[serde(default)]
    pub when_not_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncounterIr {
    pub id: String,
    #[serde(default)]
    pub title: String,
    pub text: String,
    pub choices: Vec<ChoiceIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChoiceIr {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub delta: std::collections::BTreeMap<String, i32>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
    /// Relics to add (union with the carried set). Does not replace.
    /// Use `heritage_remove` to drop an item.
    #[serde(default)]
    pub heritage: Vec<String>,
    #[serde(default)]
    pub heritage_remove: Vec<String>,
    #[serde(default)]
    pub next_heir: Option<String>,
    #[serde(default)]
    pub next_heir_name: Option<String>,
    #[serde(default)]
    pub when_flag: Option<String>,
    #[serde(default)]
    pub when_not_flag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebriefIr {
    #[serde(default)]
    pub text: String,
    pub fiches: Vec<FicheIr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FicheIr {
    pub id: String,
    pub nom: String,
    pub texte: String,
    #[serde(default)]
    pub when_flag: Option<String>,
    #[serde(default)]
    pub when_not_flag: Option<String>,
}

impl Ir {
    pub fn from_json(bytes: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(bytes)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err(format!("expected {}, got {}", SCHEMA, self.schema));
        }
        if self.chapter.petitions.is_empty() {
            return Err("chapter has no petitions".into());
        }
        if self.chapter.set_piece.choices.is_empty() {
            return Err("set_piece has no choices".into());
        }
        let known_money: BTreeSet<&str> = self
            .chapter
            .monnaies
            .iter()
            .map(|c| c.id.as_str())
            .collect();
        let mut petition_ids = BTreeSet::new();
        for p in &self.chapter.petitions {
            if p.choices.is_empty() {
                return Err(format!("petition {} has no choices", p.id));
            }
            if !petition_ids.insert(p.id.as_str()) {
                return Err(format!("petition id dupliqué : {}", p.id));
            }
            for c in &p.choices {
                for k in c.delta.keys() {
                    if !known_money.contains(k.as_str()) {
                        return Err(format!(
                            "petition {} / {} : delta `{k}` sur une monnaie non déclarée",
                            p.id, c.id
                        ));
                    }
                }
            }
        }
        for c in &self.chapter.set_piece.choices {
            for k in c.delta.keys() {
                if !known_money.contains(k.as_str()) {
                    return Err(format!(
                        "set_piece / {} : delta `{k}` sur une monnaie non déclarée",
                        c.id
                    ));
                }
            }
        }
        let mut lieu_ids = BTreeSet::new();
        for lieu in &self.chapter.lieux_jouables {
            if !lieu_ids.insert(lieu.id.as_str()) {
                return Err(format!("lieu jouable id dupliqué : {}", lieu.id));
            }
            for pid in &lieu.petitions {
                if !petition_ids.contains(pid.as_str()) {
                    return Err(format!("lieu {} : petition `{pid}` inconnue", lieu.id));
                }
            }
        }
        Ok(())
    }
}
