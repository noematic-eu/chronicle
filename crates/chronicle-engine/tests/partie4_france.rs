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
fn guerre_longue_to_linteau() {
    let camp = camp();
    let c1 = camp.get("deux-heritiers").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "guillaume");
    let s1 = play_ids(c1, state, &["loger-thomas", "rire-salique", "peage-local"]);
    assert!(s1.flags.contains("thomas-loge"));
    assert!(s1.flags.contains("camp-local"));

    let c2 = camp.get("chevauchee-peste").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "mahaut");
    let s2 = play_ids(
        c2,
        state,
        &[
            "ouvrir-fuyards",
            "payer-rancon",
            "payer-fosses",
            "rayer-livre",
        ],
    );
    assert!(s2.flags.contains("morts-rayes"));

    let c3 = camp.get("jacques").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "colin");
    let s3 = play_ids(c3, state, &["payer-jean", "cacher-herve", "vin-deux"]);
    assert!(s3.flags.contains("vin-deux-camps"));
    assert!(!s3.flags.contains("pique-prise"));

    let c4 = camp.get("deux-rois-une-fille").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s3))).unwrap();
    assert_eq!(state.heir.id, "colette");
    let s4 = play_ids(c4, state, &["salle", "loger-deux", "rien-clouer"]);
    assert!(s4.flags.contains("pas-clou"));
    assert_eq!(s4.next_chapter.as_deref(), Some("placard"));
}
