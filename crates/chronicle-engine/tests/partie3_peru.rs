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

fn through_deux_incas() -> WorldState {
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
    let s3 = play_ids(c3, state, &["livrer", "donner-fils", "ouvrir"]);
    let c4 = camp.get("tambo-mita").unwrap();
    let (state, _) = boot_with(c4, Some(&Carry::from_state(&s3))).unwrap();
    let s4 = play_ids(c4, state, &["vrai", "rendre-lama", "envoyer-frere"]);
    let c5 = camp.get("celle-qui-reste").unwrap();
    let (state, _) = boot_with(c5, Some(&Carry::from_state(&s4))).unwrap();
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
    let c6 = camp.get("deux-incas").unwrap();
    let (state, _) = boot_with(c6, Some(&Carry::from_state(&s5))).unwrap();
    play_ids(c6, state, &["cuzco", "suivre-illa", "nourrir-deux"])
}

#[test]
fn barbes_to_1572() {
    let camp = camp();
    let s6 = through_deux_incas();
    assert!(s6.flags.contains("nouvelle-cajamarca"));

    let c7 = camp.get("cajamarca").unwrap();
    let (state, _) = boot_with(c7, Some(&Carry::from_state(&s6))).unwrap();
    assert_eq!(state.heir.id, "waman");
    let s7 = play_ids(
        c7,
        state,
        &[
            "envoyer-lamas",
            "cacher-blesse",
            "loger-interprete",
            "offrir-magasin",
        ],
    );
    assert!(s7.flags.contains("magasin-offert"));

    let c8 = camp.get("encomienda").unwrap();
    let (state, _) = boot_with(c8, Some(&Carry::from_state(&s7))).unwrap();
    assert_eq!(state.heir.id, "magdalena");
    let s8 = play_ids(
        c8,
        state,
        &[
            "baptiser-murer",
            "tribut-grain",
            "taire-taqui",
            "encore",
            "mot-couvent",
            "chapelle",
        ],
    );
    assert!(s8.flags.contains("conopa-muree"));
    assert!(s8.flags.contains("nom-chretien"));
    assert!(s8.flags.contains("mot-au-couvent"));

    let c9 = camp.get("vilcabamba").unwrap();
    let (state, _) = boot_with(c9, Some(&Carry::from_state(&s8))).unwrap();
    assert_eq!(state.heir.id, "diego");
    let s9 = play_ids(c9, state, &["puna-fuyard", "cacher-hommes", "huaca-muree"]);
    assert!(s9.flags.contains("amaru-note"));
    assert_eq!(s9.next_chapter.as_deref(), Some("toledo"));
}
