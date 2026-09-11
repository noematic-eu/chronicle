use chronicle_dsl::{compile_campaign, stories_dir};
use chronicle_engine::{apply, boot, boot_with, choice_ids, Carry, Command, Ir, WorldState};

fn camp() -> chronicle_dsl::Campaign {
    compile_campaign(&stories_dir().join("silence/play")).unwrap()
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
fn eau_et_chapelle() {
    let camp = camp();
    let c1 = camp.get("chemin-ordinaire").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["refuser-lot", "chasser", "garder-pain", "volets"]);
    assert!(!s1.flags.contains("cache-pretre"));

    let c2 = camp.get("neuf-colonnes").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "jean");
    let s2 = play_ids(c2, state, &["donner-eau", "ouvrir", "ramasser", "fosses"]);
    assert!(s2.flags.contains("chapelet"));
    assert!(s2.heritage.contains("chapelet"));
    assert!(s2.flags.contains("vu-fosses-1794"));

    let c3 = camp.get("le-nom").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "louis");
    let s3 = play_ids(
        c3,
        state,
        &["laisser-prier", "dire-silence", "vendre-fleurs", "garder-tiroir", "messe"],
    );
    assert!(s3.flags.contains("dit-silence"));
    assert!(s3.flags.contains("vu-chapelle"));
    assert!(s3.next_chapter.is_none());
}
