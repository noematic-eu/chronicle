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
fn etat_pain_to_grele() {
    let camp = camp();
    let c1 = camp.get("fronde").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "nicolas");
    let s1 = play_ids(c1, state, &["etable", "loger-roi", "ouvrir-nourris"]);
    assert!(s1.flags.contains("parti-roi"));
    assert!(s1.flags.contains("lieu-etable"));

    let c2 = camp.get("sel-dragons").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "louis-aurel");
    let s2 = play_ids(c2, state, &["bois", "faux-sel", "payer-intendant"]);
    assert!(s2.flags.contains("faux-saunage"));
    assert!(s2.flags.contains("intendant-paye"));

    let c3 = camp.get("zinc-grele").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "claire");
    let s3 = play_ids(c3, state, &["cafe", "lire-haut", "raturer"]);
    assert!(s3.flags.contains("opinion-zinc"));
    assert!(s3.flags.contains("annee-raturee"));
    assert_eq!(s3.heir.id, "antoine");
    assert!(s3.next_chapter.is_none());
}
