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
fn wulfgar_through_part1() {
    let camp = camp();
    let c1 = camp.get("dernier-relais").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["oui", "relacher", "garder", "loger"]);
    assert_eq!(s1.heir.id, "wulfgar");
    assert!(s1.flags.contains("wulfgar"));

    let c2 = camp.get("bapteme").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "wulfgar");
    assert!(state.flags.contains("prev-heir-wulfgar"));
    assert!(state.narrative.contains("Wulfgar"));
    let s2 = play_ids(
        c2,
        state,
        &["cacher", "abriter", "garder-chevaux", "eglise"],
    );
    assert!(s2.flags.contains("vu-rite"));
    assert!(s2.heritage.contains("fibule"));

    let c3 = camp.get("loi-des-fils").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert_eq!(state.heir.id, "leonce");
    assert!(state.flags.contains("prev-heir-wulfgar"));
    let s3 = play_ids(c3, state, &["couper", "non", "cite", "duel"]);
    assert_eq!(s3.heir.id, "leonce");
    assert!(s3.flags.contains("ragenar-bois"));
    assert!(s3.fiches.iter().any(|f| f.id == "institution:loi-des-fils"));
    assert!(s3.next_chapter.is_none());
}

#[test]
fn aurel_skips_sang_petition() {
    let camp = camp();
    let c1 = camp.get("dernier-relais").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["non", "prelever", "vendre", "refuser"]);
    assert_eq!(s1.heir.id, "aurel");

    let c2 = camp.get("bapteme").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    assert!(state.flags.contains("no-horse"));
    let s2 = play_ids(c2, state, &["jeter", "chasser", "regarder", "gue"]);
    assert!(!s2.heritage.contains("fibule"));
    assert!(s2.flags.contains("vu-armee"));

    let c3 = camp.get("loi-des-fils").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    assert!(state.flags.contains("prev-heir-aurel"));
    assert!(!visible_sang(&state, c3));
    let s3 = play_ids(c3, state, &["aine", "oui", "ceder-cle"]);
    assert_eq!(s3.heir.id, "ragenar");
}

fn visible_sang(state: &WorldState, ir: &Ir) -> bool {
    ir.chapter
        .petitions
        .iter()
        .any(|p| p.id == "sang" && state.flags.contains("prev-heir-wulfgar"))
}
