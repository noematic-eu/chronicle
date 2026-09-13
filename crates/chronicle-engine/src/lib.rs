//! Chronicle engine: chapter mode machine, no I/O.

mod command;
mod engine;
mod error;
mod frame;
mod ir;
mod state;

pub use command::{Command, PersistHint};
pub use engine::{apply, boot, boot_with, choice_ids, render, Step};
pub use error::{EngineError, Result};
pub use frame::{plain_em, Frame, Verb};
pub use ir::{
    ChapterIr, ChoiceIr, ChronicleMeta, CurrencyIr, DebriefIr, EncounterIr, FicheIr, HeirIr, Ir,
    LieuJouable, PetitionIr, TextBlock, SCHEMA,
};
pub use state::{Carry, FicheGot, HeirState, WorldState, NOTES_CAP};
