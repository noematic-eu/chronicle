use chronicle_dsl::{compile_campaign, stories_dir};
use chronicle_engine::{apply, boot, boot_with, choice_ids, Carry, Command, Ir, WorldState};

fn camp() -> chronicle_dsl::Campaign {
    compile_campaign(&stories_dir().join("france/play")).unwrap()
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
fn nation_cahier_to_1814() {
    let camp = camp();
    let c1 = camp.get("cahier").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "antoine");
    let s1 = play_ids(
        c1,
        state,
        &[
            "cafe",
            "ecrire-gabelle",
            "signer-antoine",
            "ecrire-grele",
            "abattre",
        ],
    );
    assert!(s1.flags.contains("mention-gabelle"));
    assert!(s1.flags.contains("plume-antoine"));
    assert!(s1.flags.contains("abattre-symbole"));
    assert!(s1.heritage.contains("cle"));
    assert!(s1.heritage.contains("cahier-signe"));
    assert_eq!(s1.next_chapter.as_deref(), Some("an-93"));

    let c2 = camp.get("an-93").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "antoine");
    assert!(state.heritage.contains("cahier-signe"));
    let s2 = play_ids(
        c2,
        state,
        &["cafe", "enseigne-nation", "prendre-assignat", "tordre-sel"],
    );
    assert!(s2.flags.contains("enseigne-nation"));
    assert!(s2.flags.contains("sel-tordu"));
    assert!(s2.flags.contains("antoine-vit"));
    assert!(!s2.flags.contains("antoine-liste"));
    assert_eq!(s2.heir.id, "elise");
    assert!(s2.heritage.contains("cahier-signe"));
    assert_eq!(s2.next_chapter.as_deref(), Some("fils-et-route"));

    let c3 = camp.get("fils-et-route").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "elise");
    assert!(state.flags.contains("prev-heir-elise"));
    let s3 = play_ids(c3, state, &["place", "laisser-sort", "aigle"]);
    assert!(s3.flags.contains("fils-parti"));
    assert!(s3.flags.contains("cocarde-aigle"));
    assert!(s3.heritage.contains("cocarde"));
    assert!(s3.heritage.contains("cahier-signe"));
    assert!(s3.heritage.contains("cle"));
    assert_eq!(s3.next_chapter.as_deref(), Some("trois-jours"));
}
