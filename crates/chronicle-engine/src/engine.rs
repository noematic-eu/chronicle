use crate::command::{Command, PersistHint};
use crate::error::{EngineError, Result};
use crate::frame::{Frame, Verb};
use crate::ir::{ChoiceIr, EncounterIr, Ir, PetitionIr};
use crate::state::{Carry, FicheGot, HeirState, WorldState, NOTES_CAP};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub struct Step {
    pub state: WorldState,
    pub frame: Frame,
    pub persist: PersistHint,
    pub halt: bool,
}

pub fn boot(ir: &Ir) -> Result<(WorldState, Frame)> {
    boot_with(ir, None)
}

pub fn boot_with(ir: &Ir, carry: Option<&Carry>) -> Result<(WorldState, Frame)> {
    ir.validate().map_err(EngineError::InvalidIr)?;
    let ch = &ir.chapter;
    let mut monnaies = BTreeMap::new();
    for c in &ch.monnaies {
        monnaies.insert(c.id.clone(), c.start);
    }
    let mut flags = BTreeSet::new();
    let mut heritage: BTreeSet<String> = ch.heritage_default.iter().cloned().collect();
    let mut notes = Vec::new();
    let mut heir = HeirState {
        id: ch.heir.id.clone(),
        name: ch.heir.name.clone(),
        title: ch.heir.title.clone(),
    };
    if let Some(c) = carry {
        flags = c.flags.clone();
        if let Some(prev) = &c.heir {
            flags.insert(format!("prev-heir-{}", prev.id));
        }
        if !c.heritage.is_empty() {
            heritage = c.heritage.clone();
        }
        notes = c.notes.clone();
        for (k, v) in &c.monnaies {
            if monnaies.contains_key(k) {
                monnaies.insert(k.clone(), *v);
            }
        }
        if ch.heir_policy != "chapter" {
            if let Some(prev) = &c.heir {
                heir = prev.clone();
            }
        }
    }
    let mut state = WorldState {
        chapter: ch.id.clone(),
        mode: "briefing".into(),
        petition_i: 0,
        cursor: 0,
        monnaies,
        flags,
        notes,
        heritage,
        fiches: Vec::new(),
        heir,
        narrative: String::new(),
        last_choice: None,
        next_chapter: ch.next.clone(),
        overlay: None,
        lieu: None,
    };
    state.narrative = render_briefing(ir, &state);
    let frame = render(ir, &state)?;
    Ok((state, frame))
}

pub fn apply(ir: &Ir, mut state: WorldState, cmd: Command) -> Result<Step> {
    let mut persist = PersistHint::Skip;
    let mut halt = false;
    let overlay = state.overlay.clone();

    if overlay.is_some() {
        match cmd {
            Command::Close | Command::Notes | Command::Help | Command::Continue => {
                state.overlay = None;
            }
            Command::Quit => halt = true,
            _ => {}
        }
        let frame = render(ir, &state)?;
        return Ok(Step {
            state,
            frame,
            persist,
            halt,
        });
    }

    match cmd {
        Command::MoveCursor { delta } => {
            let n = list_len(ir, &state);
            if n > 0 {
                let next = state.cursor as i32 + delta;
                state.cursor = next.rem_euclid(n as i32) as usize;
            }
        }
        Command::Select { index } => {
            let n = list_len(ir, &state);
            if n > 0 {
                state.cursor = (index as usize).min(n - 1);
            }
        }
        Command::Choose { index } => {
            choose(ir, &mut state, index as usize)?;
            persist = PersistHint::Flush;
        }
        Command::Continue => {
            continue_mode(ir, &mut state)?;
            persist = PersistHint::Flush;
            if state.mode == "done" {
                halt = true;
            }
        }
        Command::Notes => state.overlay = Some("notes".into()),
        Command::Help => state.overlay = Some("help".into()),
        Command::Close => {}
        Command::Quit => halt = true,
    }

    let frame = render(ir, &state)?;
    Ok(Step {
        state,
        frame,
        persist,
        halt,
    })
}

fn flag_ok(state: &WorldState, when: &Option<String>, when_not: &Option<String>) -> bool {
    if let Some(f) = when {
        if !state.flags.contains(f) {
            return false;
        }
    }
    if let Some(f) = when_not {
        if state.flags.contains(f) {
            return false;
        }
    }
    true
}

fn visible_lieux<'a>(ir: &'a Ir, state: &WorldState) -> Vec<&'a crate::ir::LieuJouable> {
    ir.chapter
        .lieux_jouables
        .iter()
        .filter(|l| flag_ok(state, &l.when_flag, &l.when_not_flag))
        .collect()
}

fn visible_petitions<'a>(ir: &'a Ir, state: &WorldState) -> Vec<(usize, &'a PetitionIr)> {
    let allowed: Option<Vec<String>> = state.lieu.as_ref().and_then(|id| {
        ir.chapter
            .lieux_jouables
            .iter()
            .find(|l| &l.id == id)
            .map(|l| l.petitions.clone())
    });
    ir.chapter
        .petitions
        .iter()
        .enumerate()
        .filter(|(_, p)| flag_ok(state, &p.when_flag, &p.when_not_flag))
        .filter(|(_, p)| match &allowed {
            None => true,
            Some(ids) => ids.iter().any(|id| id == &p.id),
        })
        .collect()
}

fn visible_choices<'a>(choices: &'a [ChoiceIr], state: &WorldState) -> Vec<&'a ChoiceIr> {
    choices
        .iter()
        .filter(|c| flag_ok(state, &c.when_flag, &c.when_not_flag))
        .collect()
}

pub fn choice_ids(ir: &Ir, state: &WorldState) -> Vec<String> {
    match state.mode.as_str() {
        "lieu" => visible_lieux(ir, state)
            .into_iter()
            .map(|l| l.id.clone())
            .collect(),
        "petition" => ir
            .chapter
            .petitions
            .get(state.petition_i)
            .map(|p| {
                visible_choices(&p.choices, state)
                    .into_iter()
                    .map(|c| c.id.clone())
                    .collect()
            })
            .unwrap_or_default(),
        "set_piece" => visible_choices(&ir.chapter.set_piece.choices, state)
            .into_iter()
            .map(|c| c.id.clone())
            .collect(),
        _ => Vec::new(),
    }
}

fn list_len(ir: &Ir, state: &WorldState) -> usize {
    choice_ids(ir, state).len()
}

fn enter_petition(ir: &Ir, state: &mut WorldState, after: Option<usize>) {
    let next = visible_petitions(ir, state)
        .into_iter()
        .map(|(i, _)| i)
        .find(|i| match after {
            None => true,
            Some(a) => *i > a,
        });
    if let Some(i) = next {
        state.mode = "petition".into();
        state.petition_i = i;
        state.cursor = 0;
        state.narrative = render_petition(ir, state, &ir.chapter.petitions[i]);
    } else {
        enter_set_piece(ir, state);
    }
}

fn enter_lieu(ir: &Ir, state: &mut WorldState) {
    let lieux = visible_lieux(ir, state);
    if lieux.is_empty() {
        enter_petition(ir, state, None);
        return;
    }
    state.mode = "lieu".into();
    state.cursor = 0;
    state.narrative = interpolate(
        ir,
        state,
        "Tu ne peux pas être partout. Où vas-tu ? Le reste de cette France-là se fera sans toi.",
    );
}

fn continue_mode(ir: &Ir, state: &mut WorldState) -> Result<()> {
    match state.mode.as_str() {
        "briefing" => {
            if ir.chapter.lieux_jouables.is_empty() {
                enter_petition(ir, state, None);
            } else {
                enter_lieu(ir, state);
            }
        }
        "aftermath" => {
            enter_petition(ir, state, Some(state.petition_i));
        }
        "set_aftermath" => {
            enter_debrief(ir, state);
        }
        "debrief" => {
            state.mode = "done".into();
            let mut text = format!(
                "La chronique s'arrête ici pour ce siècle.\n\nHéritier suivant : {}.",
                state.heir.name
            );
            if let Some(n) = state.next_chapter.as_deref() {
                text.push_str(&format!("\nChapitre suivant : {n}."));
            }
            state.narrative = text;
        }
        "done" => {}
        other => {
            return Err(EngineError::WrongMode("Continue", other.into()));
        }
    }
    Ok(())
}

fn enter_set_piece(ir: &Ir, state: &mut WorldState) {
    state.mode = "set_piece".into();
    state.cursor = 0;
    state.narrative = render_set_piece(ir, state, &ir.chapter.set_piece);
}

fn enter_debrief(ir: &Ir, state: &mut WorldState) {
    state.mode = "debrief".into();
    state.fiches = earned_fiches(ir, state);
    if state.heritage.is_empty() {
        state.heritage = ir.chapter.heritage_default.iter().cloned().collect();
    }
    state.narrative = render_debrief(ir, state);
}

fn choose(ir: &Ir, state: &mut WorldState, index: usize) -> Result<()> {
    if state.mode == "lieu" {
        let lieux = visible_lieux(ir, state);
        let lieu = lieux
            .get(index)
            .ok_or_else(|| EngineError::UnknownChoice(index.to_string()))?;
        state.lieu = Some(lieu.id.clone());
        state.flags.insert(format!("lieu-{}", lieu.id));
        state.cursor = 0;
        let body = interpolate(ir, state, &lieu.texte);
        state.narrative = if body.is_empty() {
            lieu.nom.clone()
        } else {
            format!("{{em}}{}{{/em}}\n\n{}", lieu.nom, body)
        };
        enter_petition(ir, state, None);
        return Ok(());
    }
    let (choice, after_mode) = match state.mode.as_str() {
        "petition" => {
            let p = ir
                .chapter
                .petitions
                .get(state.petition_i)
                .ok_or_else(|| EngineError::Other("no petition".into()))?;
            let vis = visible_choices(&p.choices, state);
            let c = vis
                .get(index)
                .ok_or_else(|| EngineError::UnknownChoice(index.to_string()))?;
            ((*c).clone(), "aftermath")
        }
        "set_piece" => {
            let vis = visible_choices(&ir.chapter.set_piece.choices, state);
            let c = vis
                .get(index)
                .ok_or_else(|| EngineError::UnknownChoice(index.to_string()))?;
            ((*c).clone(), "set_aftermath")
        }
        other => return Err(EngineError::WrongMode("Choose", other.into())),
    };
    apply_choice(ir, state, &choice);
    state.mode = after_mode.into();
    state.cursor = 0;
    state.last_choice = Some(choice.id.clone());
    state.narrative = interpolate(ir, state, &choice.text);
    if state.narrative.is_empty() {
        state.narrative = choice.label.clone();
    }
    Ok(())
}

fn apply_choice(ir: &Ir, state: &mut WorldState, choice: &ChoiceIr) {
    for (k, d) in &choice.delta {
        if let Some(e) = state.monnaies.get_mut(k) {
            *e = (*e + d).max(0);
        }
    }
    for f in &choice.flags {
        state.flags.insert(f.clone());
    }
    if let Some(note) = &choice.notes {
        if state.notes.len() < NOTES_CAP {
            state.notes.push(note.clone());
        }
    }
    // `heritage` is a union: listing an item never wipes the rest of the cave.
    for h in &choice.heritage {
        state.heritage.insert(h.clone());
    }
    if ir.chapter.cheval_si_chevaux
        && !choice.heritage.is_empty()
        && state.monnaies.get("chevaux").copied().unwrap_or(0) > 0
    {
        state.heritage.insert("cheval".into());
    }
    for h in &choice.heritage_remove {
        state.heritage.remove(h);
    }
    if let Some(id) = &choice.next_heir {
        state.heir.id = id.clone();
        if let Some(name) = &choice.next_heir_name {
            state.heir.name = name.clone();
        }
        state.heir.title.clear();
    }
}

fn earned_fiches(ir: &Ir, state: &WorldState) -> Vec<FicheGot> {
    ir.chapter
        .debrief
        .fiches
        .iter()
        .filter(|f| {
            if let Some(flag) = &f.when_flag {
                if !state.flags.contains(flag) {
                    return false;
                }
            }
            if let Some(flag) = &f.when_not_flag {
                if state.flags.contains(flag) {
                    return false;
                }
            }
            true
        })
        .map(|f| FicheGot {
            id: f.id.clone(),
            nom: f.nom.clone(),
            texte: f.texte.clone(),
        })
        .collect()
}

fn interpolate(ir: &Ir, state: &WorldState, text: &str) -> String {
    let mut out = text.to_string();
    out = out.replace("{heir.name}", &state.heir.name);
    out = out.replace("{heir.title}", &state.heir.title);
    out = out.replace("{heir.id}", &state.heir.id);
    out = out.replace("{chapter.title}", &ir.chapter.title);
    out = out.replace("{clock.dates}", &ir.chapter.dates);
    for (k, v) in &state.monnaies {
        out = out.replace(&format!("{{{k}}}"), &v.to_string());
    }
    out
}

fn render_briefing(ir: &Ir, state: &WorldState) -> String {
    let b = &ir.chapter.briefing;
    let body = interpolate(ir, state, &b.text);
    if b.masthead.is_empty() {
        body
    } else {
        format!("{}\n\n{}", interpolate(ir, state, &b.masthead), body)
    }
}

fn render_petition(ir: &Ir, state: &WorldState, p: &PetitionIr) -> String {
    let head = if p.title.is_empty() {
        String::new()
    } else {
        format!("{{em}}{}{{/em}}\n\n", p.title)
    };
    format!("{}{}", head, interpolate(ir, state, &p.text))
}

fn render_set_piece(ir: &Ir, state: &WorldState, sp: &EncounterIr) -> String {
    let head = if sp.title.is_empty() {
        String::new()
    } else {
        format!("{{em}}{}{{/em}}\n\n", sp.title)
    };
    format!("{}{}", head, interpolate(ir, state, &sp.text))
}

fn render_debrief(ir: &Ir, state: &WorldState) -> String {
    let mut out = interpolate(ir, state, &ir.chapter.debrief.text);
    if !out.is_empty() {
        out.push_str("\n\n");
    }
    out.push_str("{em}Fiches{/em}\n");
    if state.fiches.is_empty() {
        out.push_str("Rien classé.\n");
    } else {
        for f in &state.fiches {
            out.push_str(&format!("• {} — {}\n", f.nom, f.texte));
        }
    }
    out.push('\n');
    out.push_str("{em}Reliques{/em}\n");
    if state.heritage.is_empty() {
        out.push_str("La cave est vide.\n");
    } else {
        for h in &state.heritage {
            out.push_str(&format!("• {h}\n"));
        }
    }
    if !state.notes.is_empty() {
        out.push_str("\n{em}Carnet{/em}\n");
        for n in &state.notes {
            out.push_str(&format!("• {n}\n"));
        }
    }
    out
}

fn hud_line(ir: &Ir, state: &WorldState) -> Value {
    let money = ir
        .chapter
        .monnaies
        .iter()
        .map(|c| {
            let v = state.monnaies.get(&c.id).copied().unwrap_or(0);
            format!("{} {}", c.label, v)
        })
        .collect::<Vec<_>>()
        .join("   ");
    let phase = match state.play_mode() {
        "briefing" => "briefing".into(),
        "lieu" => "où".into(),
        "petition" => format!(
            "pétition {}/{}",
            state.petition_i + 1,
            ir.chapter.petitions.len()
        ),
        "aftermath" => format!(
            "après {}/{}",
            state.petition_i + 1,
            ir.chapter.petitions.len()
        ),
        "set_piece" => "set-piece".into(),
        "set_aftermath" => "après set-piece".into(),
        "debrief" => "debrief".into(),
        "done" => "fin".into(),
        "notes" => "carnet".into(),
        "help" => "aide".into(),
        other => other.into(),
    };
    let location = state
        .lieu
        .as_ref()
        .and_then(|id| {
            ir.chapter
                .lieux_jouables
                .iter()
                .find(|l| &l.id == id)
                .map(|l| l.nom.clone())
        })
        .unwrap_or_else(|| ir.chronicle.lieu.clone());
    json!({
        "location": location,
        "heir": state.heir.name,
        "phase": phase,
        "money": money,
    })
}

fn globals() -> Vec<Verb> {
    vec![
        Verb::global("notes", "n"),
        Verb::global("help", "?"),
        Verb::global("quit", "q"),
    ]
}

pub fn render(ir: &Ir, state: &WorldState) -> Result<Frame> {
    if let Some(kind) = &state.overlay {
        return render_overlay(ir, state, kind);
    }
    match state.mode.as_str() {
        "briefing" | "aftermath" | "set_aftermath" | "debrief" | "done" => {
            render_page(ir, state, "continue")
        }
        "petition" => render_list(ir, state, petition_list(ir, state)),
        "lieu" => render_list(ir, state, lieu_list(ir, state)),
        "set_piece" => render_list(ir, state, set_piece_list(ir, state)),
        other => Err(EngineError::Other(format!("no renderer for {other}"))),
    }
}

fn render_page(ir: &Ir, state: &WorldState, continue_id: &str) -> Result<Frame> {
    let mut verbs = vec![Verb::local(continue_id, "enter")];
    verbs.extend(globals());
    let title = match state.mode.as_str() {
        "briefing" => "briefing",
        "aftermath" | "set_aftermath" => "suite",
        "debrief" => "chronique",
        "done" => "transmission",
        other => other,
    };
    Ok(Frame {
        mode: state.mode.clone(),
        slots: json!({
            "hud": hud_line(ir, state),
            "narrative": state.narrative,
            "list": [],
            "selected": 0,
            "title": title,
        }),
        verbs,
    }
    .with_footer())
}

fn render_list(ir: &Ir, state: &WorldState, list: Vec<Value>) -> Result<Frame> {
    let mut verbs = vec![
        Verb::local("up", "k"),
        Verb::local("down", "j"),
        Verb::local("choose", "enter"),
    ];
    verbs.extend(globals());
    Ok(Frame {
        mode: state.mode.clone(),
        slots: json!({
            "hud": hud_line(ir, state),
            "narrative": state.narrative,
            "list": list,
            "selected": state.cursor,
            "title": state.mode,
        }),
        verbs,
    }
    .with_footer())
}

fn lieu_list(ir: &Ir, state: &WorldState) -> Vec<Value> {
    visible_lieux(ir, state)
        .into_iter()
        .map(|l| json!({ "id": l.id, "name": l.nom }))
        .collect()
}

fn petition_list(ir: &Ir, state: &WorldState) -> Vec<Value> {
    ir.chapter
        .petitions
        .get(state.petition_i)
        .map(|p| {
            visible_choices(&p.choices, state)
                .into_iter()
                .map(|c| json!({ "id": c.id, "name": c.label }))
                .collect()
        })
        .unwrap_or_default()
}

fn set_piece_list(ir: &Ir, state: &WorldState) -> Vec<Value> {
    visible_choices(&ir.chapter.set_piece.choices, state)
        .into_iter()
        .map(|c| json!({ "id": c.id, "name": c.label }))
        .collect()
}

fn render_overlay(ir: &Ir, state: &WorldState, kind: &str) -> Result<Frame> {
    let narrative = match kind {
        "notes" => {
            if state.notes.is_empty() {
                "Le carnet est vide.".into()
            } else {
                state
                    .notes
                    .iter()
                    .enumerate()
                    .map(|(i, n)| format!("{}. {n}", i + 1))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
        "help" => {
            "j/k  liste\nEnter  choisir / continuer\nn  carnet\n?  aide\nq  quitter\n\nParfois tu choisis d'abord un lieu : tu ne peux pas être partout. L'histoire continue si tu rates.".into()
        }
        _ => String::new(),
    };
    let mut verbs = vec![Verb::local("close", "x")];
    verbs.extend(globals());
    Ok(Frame {
        mode: kind.into(),
        slots: json!({
            "hud": hud_line(ir, state),
            "narrative": narrative,
            "overlay_title": kind,
            "list": [],
            "selected": 0,
        }),
        verbs,
    }
    .with_footer())
}
