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
fn republique_to_aout() {
    let camp = camp();
    let c1 = camp.get("trois-jours").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "victor");
    let s1 = play_ids(
        c1,
        state,
        &["place", "crier-orleans", "chantier-oui", "servir-barricade"],
    );
    assert!(s1.flags.contains("orleans-cri"));
    assert!(s1.flags.contains("cote-barricade"));
    assert!(s1.heritage.contains("cote-barricade"));
    assert_eq!(s1.heir.id, "louise");
    assert_eq!(s1.next_chapter.as_deref(), Some("prussiens"));

    let c2 = camp.get("prussiens").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "louise");
    let s2 = play_ids(c2, state, &["place", "payer-amende", "servir-soupe"]);
    assert!(s2.flags.contains("amende-payee"));
    assert!(s2.flags.contains("louche-honte"));
    assert!(s2.heritage.contains("louche"));
    assert_eq!(s2.heir.id, "henri");
    assert_eq!(s2.next_chapter.as_deref(), Some("affaire"));

    let c3 = camp.get("affaire").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "henri");
    let s3 = play_ids(
        c3,
        state,
        &[
            "cafe",
            "pincer-innocent",
            "salle-paul",
            "poser-louche",
            "cote-famille",
        ],
    );
    assert!(s3.flags.contains("journal-innocent"));
    assert!(s3.flags.contains("dreyfus-famille"));
    assert_eq!(s3.heir.id, "jeanne");
    assert_eq!(s3.next_chapter.as_deref(), Some("aout"));

    let c4 = camp.get("aout").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s3))).unwrap();
    assert_eq!(state.heir.id, "jeanne");
    let s4 = play_ids(
        c4,
        state,
        &["cafe", "servir-bout", "crediter-14", "fibule-livre"],
    );
    assert!(s4.flags.contains("verre-bout"));
    assert!(s4.flags.contains("livre-ferme"));
    assert!(s4.heritage.contains("livre-ferme"));
    assert!(s4.heritage.contains("fibule-dans-le-livre"));
    assert!(!s4.heritage.contains("fibule"));
    assert!(s4.next_chapter.is_none());
}
