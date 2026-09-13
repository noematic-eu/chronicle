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

fn through_part1_open() -> WorldState {
    let camp = camp();
    let c1 = camp.get("murs-vides").unwrap();
    let (state, _) = boot(c1).unwrap();
    let s1 = play_ids(c1, state, &["extraire", "voler", "metal"]);
    let c2 = camp.get("seigneurs-de-colline").unwrap();
    let (state, _) = boot_with(c2, Some(&Carry::from_state(&s1))).unwrap();
    let s2 = play_ids(
        c2,
        state,
        &["garder-tampu", "donner", "razzier", "garder-metal", "tenir"],
    );
    let c3 = camp.get("hommes-du-cuzco").unwrap();
    let (state, _) = boot_with(c3, Some(&Carry::from_state(&s2))).unwrap();
    play_ids(c3, state, &["livrer", "donner-fils", "ouvrir"])
}

#[test]
fn tawantinsuyu_to_cajamarca_rumour() {
    let camp = camp();
    let s1 = through_part1_open();
    assert!(!s1.flags.contains("khipu-brule"));

    let c4 = camp.get("tambo-mita").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s1))).unwrap();
    assert_eq!(state.heir.id, "titu");
    let s4 = play_ids(c4, state, &["vrai", "rendre-lama", "envoyer-frere"]);
    assert!(s4.flags.contains("khipu-juste"));
    assert!(s4.flags.contains("frere-parti"));

    let c5 = camp.get("celle-qui-reste").unwrap();
    let (state, _) = boot_with(c5, Some(&Carry::from_state(&s4))).unwrap();
    assert_eq!(state.heir.id, "ocllo");
    let s5 = play_ids(
        c5,
        state,
        &[
            "donner-fille",
            "asseoir-otage",
            "loger-mitmaq",
            "garder-conopa",
            "dire-vrai",
        ],
    );
    assert!(s5.flags.contains("fille-aclla"));
    assert!(s5.flags.contains("otage-a-table"));
    assert!(s5.flags.contains("recensement-vrai"));

    let c6 = camp.get("deux-incas").unwrap();
    let (state, _) = boot_with(c6, Some(&Carry::from_state(&s5))).unwrap();
    assert_eq!(state.heir.id, "quispe");
    assert!(!c6
        .chapter
        .petitions
        .iter()
        .any(|p| p.id == "deficit" && state.flags.contains("khipu-truque")));
    let s6 = play_ids(c6, state, &["cuzco", "suivre-illa", "nourrir-deux"]);
    assert!(s6.flags.contains("nouvelle-cajamarca"));
    assert_eq!(s6.next_chapter.as_deref(), Some("cajamarca"));
}
