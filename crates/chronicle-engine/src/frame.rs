use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Verb {
    pub id: String,
    pub key: String,
    #[serde(skip_serializing_if = "is_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub global: bool,
}

fn is_true(v: &bool) -> bool {
    *v
}
fn is_false(v: &bool) -> bool {
    !*v
}

impl Verb {
    pub fn local(id: &str, key: &str) -> Self {
        Self {
            id: id.to_string(),
            key: key.to_string(),
            enabled: true,
            global: false,
        }
    }

    pub fn global(id: &str, key: &str) -> Self {
        Self {
            id: id.to_string(),
            key: key.to_string(),
            enabled: true,
            global: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    pub mode: String,
    pub slots: Value,
    pub verbs: Vec<Verb>,
}

impl Frame {
    pub fn footer(&self) -> String {
        let parts: Vec<String> = self
            .verbs
            .iter()
            .filter(|v| v.enabled)
            .map(|v| format!("{} {}", v.key, v.id))
            .collect();
        parts.join("   ")
    }

    pub fn with_footer(mut self) -> Self {
        let footer = self.footer();
        if let Some(obj) = self.slots.as_object_mut() {
            obj.insert("footer".into(), json!(footer));
        }
        self
    }
}

pub fn plain_em(text: &str) -> String {
    text.replace("{em}", "").replace("{/em}", "")
}
