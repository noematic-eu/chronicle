use chronicle_dsl::{compile_campaign, compile_path, Campaign};
use chronicle_engine::{boot_with, Carry, PersistHint};
use clap::{Parser, Subcommand};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

mod persist;
mod tui;

#[derive(Parser)]
#[command(
    name = "chronicle",
    about = "Chronicle player — missions in a historical spine"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Play {
        #[arg(help = "playable chapter .md or campaign directory")]
        path: PathBuf,
        #[arg(long)]
        instance: Option<PathBuf>,
        #[arg(long, help = "start this chapter id (ignore lineage next)")]
        chapter: Option<String>,
    },
    Lint {
        path: PathBuf,
    },
    Compile {
        path: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    if let Err(e) = real_main() {
        eprintln!("{e}");
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn real_main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Lint { path } => {
            if path.is_dir() {
                let camp = compile_campaign(&path).map_err(|e| e.to_string())?;
                camp.lint().map_err(|e| e.to_string())?;
                println!(
                    "ok: {} ({}) — {}",
                    camp.meta.id,
                    path.display(),
                    camp.order().join(" → ")
                );
            } else {
                let ir = compile_path(&path).map_err(|e| e.to_string())?;
                println!("ok: {} ({})", ir.chapter.id, path.display());
            }
            Ok(())
        }
        Cmd::Compile { path, output } => {
            let ir = compile_path(&path).map_err(|e| e.to_string())?;
            let json = chronicle_dsl::to_json(&ir).map_err(|e| e.to_string())?;
            std::fs::write(&output, json).map_err(|e| e.to_string())?;
            eprintln!("wrote {}", output.display());
            Ok(())
        }
        Cmd::Play {
            path,
            instance,
            chapter,
        } => {
            let inst = persist::instance_dir(instance);
            if path.is_dir() {
                let camp = compile_campaign(&path).map_err(|e| e.to_string())?;
                play_campaign(&camp, &inst, chapter)
            } else {
                let ir = compile_path(&path).map_err(|e| e.to_string())?;
                let mut camp = Campaign {
                    meta: ir.chronicle.clone(),
                    entry: ir.chapter.id.clone(),
                    chapters: std::collections::BTreeMap::new(),
                };
                camp.chapters.insert(ir.chapter.id.clone(), ir);
                play_campaign(&camp, &inst, chapter)
            }
        }
    }
}

fn play_campaign(
    camp: &Campaign,
    inst: &std::path::Path,
    force: Option<String>,
) -> Result<(), String> {
    let save_file = persist::save_path(inst, &camp.meta.id);
    let save = persist::read_lineage(inst, &camp.meta.id).map_err(|e| e.to_string())?;
    let mut chapter_id = if let Some(id) = force {
        id
    } else {
        match &save {
            None => {
                eprintln!(
                    "pas de lignée ({}) — début {}",
                    save_file.display(),
                    camp.entry
                );
                camp.entry.clone()
            }
            Some(prev) => match camp.resume(&prev.completed, prev.next_chapter.as_deref()) {
                Some(id) => {
                    eprintln!(
                        "lignée : {} · {} → {} ({})",
                        prev.heir.name,
                        prev.completed.join(", "),
                        id,
                        save_file.display()
                    );
                    id
                }
                None => {
                    eprintln!(
                        "campagne terminée : {} ({})",
                        prev.completed.join(" → "),
                        save_file.display()
                    );
                    return Ok(());
                }
            },
        }
    };
    let mut carry: Option<Carry> = save.as_ref().map(persist::to_carry);

    loop {
        let ir = camp
            .get(&chapter_id)
            .ok_or_else(|| format!("chapter `{chapter_id}` not in campaign"))?;
        let (mut state, frame) = boot_with(ir, carry.as_ref()).map_err(|e| e.to_string())?;
        if state
            .next_chapter
            .as_ref()
            .is_some_and(|n| !camp.chapters.contains_key(n))
        {
            state.next_chapter = None;
        }
        if !std::io::stdout().is_terminal() {
            println!(
                "— {} —\n{}",
                ir.chapter.title,
                frame.slots["narrative"].as_str().unwrap_or("")
            );
            return Ok(());
        }
        let final_state = tui::run(ir, state, frame, |st, hint| {
            if matches!(hint, PersistHint::Flush) && (st.mode == "debrief" || st.mode == "done") {
                let _ = persist::write_lineage(inst, &camp.meta.id, st);
            }
        })
        .map_err(|e| e.to_string())?;
        if final_state.mode == "done" || final_state.mode == "debrief" {
            persist::write_lineage(inst, &camp.meta.id, &final_state).map_err(|e| e.to_string())?;
            eprintln!(
                "lineage {}",
                persist::save_path(inst, &camp.meta.id).display()
            );
        }
        if final_state.mode != "done" {
            break;
        }
        carry = Some(Carry::from_state(&final_state));
        match final_state.next_chapter {
            Some(next) if camp.chapters.contains_key(&next) => chapter_id = next,
            _ => break,
        }
    }
    Ok(())
}
