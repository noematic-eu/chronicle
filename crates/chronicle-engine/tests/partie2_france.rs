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
fn palais_to_capet() {
    let camp = camp();
    let c1 = camp.get("ceux-qui-ne-viennent-plus").unwrap();
    let (state, _) = boot(c1).unwrap();
    assert_eq!(state.heir.id, "radegonde");
    let s1 = play_ids(c1, state, &["donner-pre", "abriter", "croire"]);
    assert!(s1.flags.contains("pre-donne"));
    assert!(s1.flags.contains("relique-doute"));
    assert!(s1.heritage.contains("relique-fragment"));
    assert!(s1.fiches.iter().any(|f| f.id == "objet:relique"));

    let c2 = camp.get("marteau-sacre").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "marten");
    let s2 = play_ids(
        c2,
        state,
        &[
            "partir",
            "chasser-fuyard",
            "acclamer",
            "via-abbe",
            "femme-cle",
        ],
    );
    assert!(s2.flags.contains("marten-parti"));
    assert!(s2.flags.contains("femme-tient"));
    assert!(s2.flags.contains("sacre-cri"));
    assert!(s2
        .fiches
        .iter()
        .any(|f| f.id == "institution:dette-de-service"));

    let c3 = camp.get("verdun-northmen-duc").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "berthe");
    let s3 = play_ids(
        c3,
        state,
        &[
            "payer-rancon",
            "otage",
            "envoyer-pain",
            "payer-dette",
            "tenir-porte",
        ],
    );
    assert!(s3.flags.contains("capet-cri"));
    assert!(s3.flags.contains("porte-tenue"));
    assert_eq!(s3.next_chapter.as_deref(), Some("chateau-contre-gue"));
}
