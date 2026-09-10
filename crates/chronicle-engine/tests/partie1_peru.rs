use chronicle_dsl::{compile_campaign, stories_dir};
use chronicle_engine::{apply, boot, boot_with, choice_ids, Carry, Command, Ir, WorldState};

fn camp() -> chronicle_dsl::Campaign {
    compile_campaign(&stories_dir().join("peru/play")).unwrap()
}

fn choose(ir: &Ir, state: WorldState, id: &str) -> WorldState {
    let list = choice_ids(ir, &state);
    let index = list
        .iter()
        .position(|s| s == id)
        .unwrap_or_else(|| panic!("choice {id} not in {list:?} (mode {})", state.mode));
    apply(ir, state, Command::Choose { index: index as u8 })
        .unwrap()
        .state
}

fn cont(ir: &Ir, state: WorldState) -> WorldState {
    apply(ir, state, Command::Continue).unwrap().state
}

fn play_ids(ir: &Ir, mut state: WorldState, ids: &[&str]) -> WorldState {
    for id in ids {
        if state.mode == "briefing" || state.mode == "aftermath" || state.mode == "set_aftermath" {
            state = cont(ir, state);
        }
        state = choose(ir, state, id);
    }
    while state.mode != "debrief" && state.mode != "done" {
        state = cont(ir, state);
        if state.mode == "petition" || state.mode == "set_piece" {
            panic!("stuck in {} with {:?}", state.mode, choice_ids(ir, &state));
        }
    }
    state
}

#[test]
fn metal_and_open_tampu() {
    let camp = camp();
    let c1 = camp.get("murs-vides").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["extraire", "voler", "metal"]);
    assert!(s1.flags.contains("metal-pille"));
    assert!(s1.heritage.contains("conopa"));

    let c2 = camp.get("seigneurs-de-colline").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "curi");
    assert!(state.flags.contains("prev-heir-rumi"));
    let s2 = play_ids(
        c2,
        state,
        &["garder-tampu", "donner", "razzier", "garder-metal", "tenir"],
    );
    assert!(s2.flags.contains("tenu-aube"));
    assert_eq!(s2.heir.id, "curi");

    let c3 = camp.get("hommes-du-cuzco").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "sinchi");
    assert!(state.flags.contains("prev-heir-curi"));
    let s3 = play_ids(c3, state, &["livrer", "donner-fils", "ouvrir"]);
    assert!(s3.flags.contains("kamayuq"));
    assert!(s3.fiches.iter().any(|f| f.id == "institution:tawantinsuyu"));
    assert!(s3.next_chapter.is_none());
}

#[test]
fn sisa_and_isolate() {
    let camp = camp();
    let c1 = camp.get("murs-vides").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["laisser", "loger", "conopa"]);
    assert!(!s1.flags.contains("metal-pille"));

    let c2 = camp.get("seigneurs-de-colline").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert!(!c2
        .chapter
        .petitions
        .iter()
        .any(|p| p.id == "metal-vu" && state.flags.contains("metal-pille")));
    let s2 = play_ids(c2, state, &["ceder", "refuser-otage", "attendre", "pukara"]);
    assert_eq!(s2.heir.id, "sisa");
    assert!(s2.flags.contains("curi-mort"));
    assert!(s2.flags.contains("tampu-au-curaca"));

    let c3 = camp.get("hommes-du-cuzco").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "sinchi");
    assert!(state.flags.contains("prev-heir-sisa"));
    let s3 = play_ids(
        c3,
        state,
        &[
            "bruler",
            "cacher-fils",
            "sisa-ouvre",
            "parler-seul",
            "tenir",
        ],
    );
    assert!(s3.flags.contains("resistance"));
    assert!(!s3.heritage.contains("khipu-ayllu"));
}
