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
fn amaru_without_being_the_center() {
    let camp = camp();
    let c1 = camp.get("alcabala").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "mateo");
    let s1 = play_ids(
        c1,
        state,
        &["couvrir-blas", "payer", "signer", "ouvrir-convoi"],
    );
    assert!(s1.flags.contains("plainte-signee"));
    assert!(s1.flags.contains("blas-couvert"));

    let cm = camp.get("messager").unwrap();
    let (state, _) = boot_with(cm, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "mateo");
    let sm = play_ids(
        cm,
        state,
        &["loger-mariano", "tomasa-seule", "blas-rien", "cave-oui"],
    );
    assert!(sm.flags.contains("mariano-loge"));
    assert!(sm.flags.contains("cave-prete"));

    let c2 = camp.get("revolte").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&sm))).unwrap();
    assert_eq!(state.heir.id, "tomasa");
    let s2 = play_ids(
        c2,
        state,
        &[
            "cave-lettre",
            "mariano-cave",
            "cacher-sud",
            "cacher-mateo",
            "puna-nuit",
        ],
    );
    assert!(s2.flags.contains("lettre-cave"));
    assert!(s2.flags.contains("fuyards-caches"));
    assert!(s2.flags.contains("cecilio-parle"));

    let c3 = camp.get("casse-kuraka").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    let s3 = play_ids(
        c3,
        state,
        &[
            "rendre-baton",
            "mur-insigne-ferme",
            "ceder-zinc",
            "genou-amer",
        ],
    );
    assert!(s3.flags.contains("sans-titre"));
    assert!(s3.flags.contains("genou"));
    assert!(s3.flags.contains("zinc-cecilio"));
    assert!(s3.next_chapter.is_none());
}
