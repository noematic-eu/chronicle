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
fn deux_cultes_to_edit() {
    let camp = camp();
    let c1 = camp.get("placard").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "pierre");
    let s1 = play_ids(c1, state, &["cacher-livre", "pluie", "francais"]);
    assert!(s1.flags.contains("livre-interdit"));
    assert!(s1.flags.contains("langue-francais"));

    let c2 = camp.get("barthelemy").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "marie");
    let s2 = play_ids(
        c2,
        state,
        &["place", "taire-jacques", "payer-olive", "dettes-nuit-gue"],
    );
    assert!(s2.flags.contains("marie-vit"));
    assert!(s2.flags.contains("nuit-sans-sang-1572"));

    let c3 = camp.get("edit").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "marie");
    let s3 = play_ids(c3, state, &["rendre-maison", "lire-tout", "servir-repas"]);
    assert!(s3.flags.contains("cave-deux-portes"));
    assert_eq!(s3.next_chapter.as_deref(), Some("fronde"));
}
