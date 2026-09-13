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
fn vice_roi_to_alcabala_table() {
    let camp = camp();
    let c1 = camp.get("toledo").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "pedro");
    let s1 = play_ids(c1, state, &["plaza", "envoyer-frere", "compter-mois"]);
    assert!(s1.flags.contains("frere-a-huancavelica"));
    assert!(s1.flags.contains("tampu-plaza"));

    let c2 = camp.get("argent-qui-passe").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "francisca");
    let s2 = play_ids(
        c2,
        state,
        &["installer", "preter", "ouvrir-lettre", "ouvrir-convoi"],
    );
    assert!(s2.flags.contains("lettre-lue"));
    assert!(s2.flags.contains("obraje"));

    let c3 = camp.get("bourbons").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "jose");
    let s3 = play_ids(c3, state, &["collecter", "loger-voyant", "lire-haut"]);
    assert!(s3.flags.contains("trop-collecte"));
    assert!(s3.flags.contains("papier-plaza"));
    assert_eq!(s3.next_chapter.as_deref(), Some("alcabala"));
}
