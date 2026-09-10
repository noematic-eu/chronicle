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
            "loi-des-fils".to_string()
        ]
    );
    assert_eq!(camp.chapters.len(), 3);
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
            "hommes-du-cuzco".to_string()
        ]
    );
    assert_eq!(camp.chapters.len(), 3);
}
