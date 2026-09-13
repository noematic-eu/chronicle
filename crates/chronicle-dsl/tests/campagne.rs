use chronicle_dsl::{compile_campaign, stories_dir};
use std::path::PathBuf;

fn play_dir(rel: &str) -> PathBuf {
    stories_dir().join(rel)
}

#[test]
fn france_partie1_order() {
    let camp = compile_campaign(&play_dir("france/play")).expect("campaign");
    assert_eq!(camp.entry, "dernier-relais");
    assert_eq!(
        camp.order(),
        vec![
            "dernier-relais".to_string(),
            "bapteme".to_string(),
            "loi-des-fils".to_string(),
            "ceux-qui-ne-viennent-plus".to_string(),
            "marteau-sacre".to_string(),
            "verdun-northmen-duc".to_string(),
            "chateau-contre-gue".to_string(),
            "preche-voeu".to_string(),
            "ceux-qui-reviennent".to_string(),
            "temple".to_string(),
            "deux-heritiers".to_string(),
            "chevauchee-peste".to_string(),
            "jacques".to_string(),
            "deux-rois-une-fille".to_string(),
            "placard".to_string(),
            "barthelemy".to_string(),
            "edit".to_string(),
            "fronde".to_string(),
            "sel-dragons".to_string(),
            "zinc-grele".to_string()
        ]
    );
    assert_eq!(camp.chapters.len(), 20);
}

#[test]
fn peru_partie1_order() {
    let camp = compile_campaign(&play_dir("peru/play")).expect("campaign");
    assert_eq!(camp.entry, "murs-vides");
    assert_eq!(camp.meta.id, "peru");
    assert_eq!(
        camp.order(),
        vec![
            "murs-vides".to_string(),
            "seigneurs-de-colline".to_string(),
            "hommes-du-cuzco".to_string(),
            "tambo-mita".to_string(),
            "celle-qui-reste".to_string(),
            "deux-incas".to_string(),
            "cajamarca".to_string(),
            "encomienda".to_string(),
            "vilcabamba".to_string(),
            "toledo".to_string(),
            "argent-qui-passe".to_string(),
            "bourbons".to_string(),
            "alcabala".to_string(),
            "messager".to_string(),
            "revolte".to_string(),
            "casse-kuraka".to_string()
        ]
    );
    assert_eq!(camp.chapters.len(), 16);
    assert_eq!(
        camp.resume(
            &[
                "murs-vides".into(),
                "seigneurs-de-colline".into(),
                "hommes-du-cuzco".into()
            ],
            None
        )
        .as_deref(),
        Some("tambo-mita")
    );
    assert_eq!(
        camp.resume(&[], Some("tambo-mita")).as_deref(),
        Some("tambo-mita")
    );
    let all = camp.order();
    assert!(camp.resume(&all, Some("cajamarca")).is_none());
}

#[test]
fn packs_lint_clean() {
    for rel in ["france/play", "peru/play", "silence/play"] {
        let camp = compile_campaign(&play_dir(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"));
        camp.lint().unwrap_or_else(|e| panic!("{rel} lint: {e}"));
    }
}

#[test]
fn lint_rejects_dangling_next() {
    let mut camp = compile_campaign(&play_dir("silence/play")).expect("silence");
    camp.chapters
        .get_mut("le-nom")
        .expect("le-nom")
        .chapter
        .next = Some("ghost".into());
    let err = camp
        .lint()
        .expect_err("dangling next must fail")
        .to_string();
    assert!(err.contains("ghost"), "{err}");
    assert!(err.contains("le-nom"), "{err}");
}

#[test]
fn silence_partie1_order() {
    let camp = compile_campaign(&play_dir("silence/play")).expect("campaign");
    assert_eq!(camp.entry, "chemin-ordinaire");
    assert_eq!(camp.meta.id, "silence");
    assert_eq!(
        camp.order(),
        vec![
            "chemin-ordinaire".to_string(),
            "neuf-colonnes".to_string(),
            "le-nom".to_string()
        ]
    );
    assert_eq!(camp.chapters.len(), 3);
}
