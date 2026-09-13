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
fn patria_cadix_to_junin() {
    let camp = camp();
    let c1 = camp.get("cadix").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "andres");
    let s1 = play_ids(
        c1,
        state,
        &["plaza", "jurer-ferdinand", "milice-oui", "tout-donner"],
    );
    assert!(s1.flags.contains("serment-ferdinand"));
    assert!(s1.flags.contains("milice-andres"));
    assert!(s1.heritage.contains("serment"));
    assert!(s1.heritage.contains("cle-corde"));
    assert_eq!(s1.next_chapter.as_deref(), Some("drapeau"));

    let c2 = camp.get("drapeau").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "andres");
    assert!(state.heritage.contains("serment"));
    let s2 = play_ids(
        c2,
        state,
        &["plaza", "enseigne-roi", "grain-roi", "mentir-roi"],
    );
    assert!(s2.flags.contains("enseigne-roi"));
    assert!(s2.flags.contains("diner-roi"));
    assert_eq!(s2.heir.id, "rosa");
    assert!(s2.heritage.contains("enseigne"));
    assert!(s2.heritage.contains("serment"));
    assert_eq!(s2.next_chapter.as_deref(), Some("junin"));

    let c3 = camp.get("junin").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "rosa");
    assert!(state.flags.contains("prev-heir-rosa"));
    let s3 = play_ids(
        c3,
        state,
        &[
            "tampu",
            "soigner-deux",
            "compter-louches",
            "brassard-patria",
        ],
    );
    assert!(s3.flags.contains("soigne-deux"));
    assert!(s3.flags.contains("brassard-patria"));
    assert!(s3.heritage.contains("louche"));
    assert!(s3.heritage.contains("serment"));
    assert!(s3.heritage.contains("cle-corde"));
    assert_eq!(s3.next_chapter.as_deref(), Some("hacienda"));
}
