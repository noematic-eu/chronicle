use chronicle_dsl::{compile_file, stories_dir};

fn playable() -> std::path::PathBuf {
    stories_dir().join("france/play/dernier-relais.md")
}

#[test]
fn dernier_relais_compiles() {
    let ir = compile_file(&playable()).expect("compile");
    assert_eq!(ir.schema, "chronicle.ir/v1");
    assert_eq!(ir.chronicle.id, "france");
    assert_eq!(ir.chapter.id, "dernier-relais");
    assert_eq!(ir.chapter.petitions.len(), 3);
    assert_eq!(ir.chapter.set_piece.choices.len(), 3);
    assert_eq!(ir.chapter.heir.id, "tetricus");
    assert!(ir.chapter.monnaies.iter().any(|c| c.id == "grain"));
}

#[test]
fn yaml_fence_required() {
    let err = chronicle_dsl::compile_markdown("# hi\n\nno fence\n", "t.md").unwrap_err();
    assert!(err.to_string().contains("no yaml fence"));
}

#[test]
fn unknown_currency_rejected() {
    let src = r#"```yaml
id: t
title: t
monnaies:
  - id: grain
    label: grain
    start: 1
heir:
  id: a
  name: A
briefing:
  text: hi
petitions:
  - id: p
    text: hi
    choices:
      - id: c
        label: x
        delta:
          corde: -1
set_piece:
  id: s
  text: hi
  choices:
    - id: z
      label: z
debrief:
  fiches: []
```
"#;
    let err = chronicle_dsl::compile_markdown(src, "t.md").unwrap_err();
    assert!(err.to_string().contains("corde"), "{}", err);
}
