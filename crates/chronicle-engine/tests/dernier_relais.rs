use chronicle_dsl::{compile_file, stories_dir};
use chronicle_engine::{apply, boot, choice_ids, Command, Ir};

fn ir() -> Ir {
    compile_file(&stories_dir().join("france/play/dernier-relais.md")).unwrap()
}

fn choose_id(
    ir: &Ir,
    state: chronicle_engine::WorldState,
    id: &str,
) -> chronicle_engine::WorldState {
    let list = choice_ids(ir, &state);
    let index = list
        .iter()
        .position(|s| s == id)
        .unwrap_or_else(|| panic!("choice {id} not in {list:?}"));
    apply(ir, state, Command::Choose { index: index as u8 })
        .unwrap()
        .state
}

#[test]
fn boot_is_briefing() {
    let ir = ir();
    let (state, frame) = boot(&ir).unwrap();
    assert_eq!(state.mode, "briefing");
    assert_eq!(frame.mode, "briefing");
    assert_eq!(state.heir.id, "tetricus");
    assert_eq!(state.monnaies["grain"], 4);
    assert!(state.narrative.contains("Tetricus"));
}

#[test]
fn loger_wulfgar_lineage() {
    let ir = ir();
    let (mut state, _) = boot(&ir).unwrap();
    state = apply(&ir, state, Command::Continue).unwrap().state;
    assert_eq!(state.mode, "petition");
    state = choose_id(&ir, state, "oui");
    assert_eq!(state.mode, "aftermath");
    assert!(state.flags.contains("xenodochium"));
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "relacher");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "garder");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    assert_eq!(state.mode, "set_piece");
    state = choose_id(&ir, state, "loger");
    assert_eq!(state.mode, "set_aftermath");
    assert!(state.flags.contains("wulfgar"));
    assert_eq!(state.heir.id, "wulfgar");
    assert_eq!(state.heir.name, "Wulfgar");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    assert_eq!(state.mode, "debrief");
    assert!(state.fiches.iter().any(|f| f.id == "fait:fin-de-l-impot"));
    assert!(state
        .fiches
        .iter()
        .any(|f| f.id == "institution:civitas-eveque"));
    assert!(state.heritage.contains("fibule"));
    assert!(state.heritage.contains("cheval"));
    let step = apply(&ir, state, Command::Continue).unwrap();
    assert!(step.halt);
    assert_eq!(step.state.mode, "done");
}

#[test]
fn refuse_burns_to_aurel() {
    let ir = ir();
    let (mut state, _) = boot(&ir).unwrap();
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "non");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "prelever");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "vendre");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    state = choose_id(&ir, state, "refuser");
    state = apply(&ir, state, Command::Continue).unwrap().state;
    assert_eq!(state.heir.id, "aurel");
    assert!(!state
        .fiches
        .iter()
        .any(|f| f.id == "institution:civitas-eveque"));
    assert!(!state.heritage.contains("cheval"));
}

#[test]
fn choose_wrong_mode() {
    let ir = ir();
    let (state, _) = boot(&ir).unwrap();
    assert!(apply(&ir, state, Command::Choose { index: 0 }).is_err());
}
