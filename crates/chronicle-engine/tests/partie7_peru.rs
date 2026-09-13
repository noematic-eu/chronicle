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
fn hacienda_to_dix_neuf() {
    let camp = camp();
    let c1 = camp.get("hacienda").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "manuel");
    let s1 = play_ids(
        c1,
        state,
        &["chacra", "vendre-bord", "envoyer-peon", "nourrir-caudillo"],
    );
    assert!(s1.flags.contains("terre-perdue"));
    assert_eq!(s1.heir.id, "carmen");
    assert_eq!(s1.next_chapter.as_deref(), Some("guano"));

    let c2 = camp.get("guano").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "carmen");
    let s2 = play_ids(
        c2,
        state,
        &[
            "tampu",
            "loger-canton",
            "crediter-peones",
            "chasser-recruteur",
        ],
    );
    assert!(s2.flags.contains("cantonais-loge"));
    assert!(s2.heritage.contains("homme-de-canton"));
    assert_eq!(s2.next_chapter.as_deref(), Some("brena"));

    let c3 = camp.get("brena").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "carmen");
    let s3 = play_ids(c3, state, &["plaza", "payer-chilienne", "servir-chiliens"]);
    assert!(s3.flags.contains("reput-1882"));
    assert!(s3.heritage.contains("louche"));
    assert_eq!(s3.heir.id, "victor");
    assert_eq!(s3.next_chapter.as_deref(), Some("gamonal"));

    let c4 = camp.get("gamonal").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s3))).unwrap();
    assert_eq!(state.heir.id, "victor");
    let s4 = play_ids(
        c4,
        state,
        &["tampu", "signer-bail", "contre-peon", "poser-louche-88"],
    );
    assert!(s4.flags.contains("bail-signe"));
    assert!(s4.heritage.contains("bail"));
    assert_eq!(s4.heir.id, "elena");
    assert_eq!(s4.next_chapter.as_deref(), Some("deux-journaux"));

    let c5 = camp.get("deux-journaux").unwrap();
    let (state, _) = boot_with(c5, Some(&Carry::from_state(&s4))).unwrap();
    assert_eq!(state.heir.id, "elena");
    let s5 = play_ids(
        c5,
        state,
        &[
            "tampu",
            "pincer-civilista",
            "salle-espagnol",
            "montrer-khipu",
            "separer-pe",
        ],
    );
    assert!(s5.flags.contains("khipu-montre"));
    assert!(s5.heritage.contains("camp-de-la-fonda"));
    assert_eq!(s5.next_chapter.as_deref(), Some("dix-neuf"));

    let c6 = camp.get("dix-neuf").unwrap();
    let (state, _) = boot_with(c6, Some(&Carry::from_state(&s5))).unwrap();
    assert_eq!(state.heir.id, "elena");
    let s6 = play_ids(
        c6,
        state,
        &["tampu", "laisser-lima", "accrocher-leguia", "poser-zinc"],
    );
    assert!(s6.flags.contains("fils-a-lima"));
    assert!(s6.flags.contains("chronique-andine"));
    assert!(s6.heritage.contains("khipu-dans-le-tiroir"));
    assert!(s6.next_chapter.is_none());
}
