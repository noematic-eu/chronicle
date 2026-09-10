#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    MoveCursor { delta: i32 },
    Select { index: u32 },
    Choose { index: u8 },
    Continue,
    Notes,
    Help,
    Close,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistHint {
    Flush,
    Skip,
}
