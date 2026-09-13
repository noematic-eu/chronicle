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
fn croix_to_temple() {
    let camp = camp();
    let c1 = camp.get("chateau-contre-gue").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "foulque");
    let s1 = play_ids(
        c1,
        state,
        &["moulin-gautier", "aller-paix", "marier", "ecrit"],
    );
    assert!(s1.flags.contains("ecrit-moulin"));
    assert!(!s1.flags.contains("moulin-perdu"));

    let c2 = camp.get("preche-voeu").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "alix");
    let s2 = play_ids(
        c2,
        state,
        &["financer", "proteger-isaac", "refuser-veuve", "garder-cle"],
    );
    assert!(s2.flags.contains("etienne-parti"));
    assert!(s2.flags.contains("livre-naissant"));

    let c3 = camp.get("ceux-qui-reviennent").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "jean");
    let s3 = play_ids(
        c3,
        state,
        &["exposer", "faire-payer", "payer-taille", "reconnaitre"],
    );
    assert!(s3.flags.contains("etienne-reconnu"));

    let c4 = camp.get("temple").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s3))).unwrap();
    assert_eq!(state.heir.id, "perrin");
    let s4 = play_ids(
        c4,
        state,
        &["bois-gerard", "preter", "rayer-ordre", "trou-choisi"],
    );
    assert!(s4.flags.contains("gerard-bois"));
    assert!(s4.flags.contains("perquis-trou"));
    assert_eq!(s4.next_chapter.as_deref(), Some("deux-heritiers"));
}
