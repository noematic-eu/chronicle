use chronicle_engine::{apply, plain_em, Command, Frame, Ir, PersistHint, WorldState};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Terminal;
use serde_json::Value;
use std::io::stdout;
use std::time::Duration;

pub fn run(
    ir: &Ir,
    mut state: WorldState,
    mut frame: Frame,
    mut on_step: impl FnMut(&WorldState, PersistHint),
) -> chronicle_engine::Result<WorldState> {
    enable_raw_mode().map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?;
    execute!(stdout(), EnterAlternateScreen)
        .map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal =
        Terminal::new(backend).map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?;
    let mut status = String::new();
    let result = loop {
        terminal
            .draw(|f| draw(f, &frame, &status))
            .map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?;
        if !event::poll(Duration::from_millis(200))
            .map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?
        {
            continue;
        }
        let Event::Key(kev) =
            event::read().map_err(|e| chronicle_engine::EngineError::Other(e.to_string()))?
        else {
            continue;
        };
        if kev.kind != KeyEventKind::Press {
            continue;
        }
        let Some(cmd) = map_key(&frame, kev) else {
            continue;
        };
        let quit = matches!(cmd, Command::Quit);
        match apply(ir, state.clone(), cmd) {
            Ok(step) => {
                on_step(&step.state, step.persist);
                state = step.state;
                frame = step.frame;
                status.clear();
                if step.halt || quit {
                    break Ok(state);
                }
            }
            Err(e) => status = e.to_string(),
        }
    };
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
    result
}

fn map_key(frame: &Frame, ev: KeyEvent) -> Option<Command> {
    if ev.modifiers.contains(KeyModifiers::CONTROL) {
        return None;
    }
    match ev.code {
        KeyCode::Esc => Some(Command::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Command::MoveCursor { delta: -1 }),
        KeyCode::Down | KeyCode::Char('j') => Some(Command::MoveCursor { delta: 1 }),
        KeyCode::Enter => {
            if verb_enabled(frame, "continue") {
                Some(Command::Continue)
            } else if verb_enabled(frame, "close") {
                Some(Command::Close)
            } else if verb_enabled(frame, "choose") {
                let idx = selected(frame);
                Some(Command::Choose {
                    index: idx.min(255) as u8,
                })
            } else {
                Some(Command::Continue)
            }
        }
        KeyCode::Char('q') => Some(Command::Quit),
        KeyCode::Char('n') => Some(Command::Notes),
        KeyCode::Char('?') => Some(Command::Help),
        KeyCode::Char('x') if verb_enabled(frame, "close") => Some(Command::Close),
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let n = c.to_digit(10).unwrap() as u8 - 1;
            if verb_enabled(frame, "choose") {
                Some(Command::Choose { index: n })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn verb_enabled(frame: &Frame, id: &str) -> bool {
    frame.verbs.iter().any(|v| v.id == id && v.enabled)
}

fn selected(frame: &Frame) -> usize {
    frame
        .slots
        .get("selected")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize
}

fn draw(f: &mut ratatui::Frame, frame: &Frame, status: &str) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(size);

    f.render_widget(
        Paragraph::new(hud_line(frame)).style(
            Style::default()
                .bg(Color::Rgb(80, 80, 128))
                .fg(Color::White),
        ),
        chunks[0],
    );

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(chunks[1]);

    let title = frame
        .slots
        .get("title")
        .or_else(|| frame.slots.get("overlay_title"))
        .and_then(Value::as_str)
        .unwrap_or(frame.mode.as_str());
    let story: Vec<Line> = plain_em(narrative(frame))
        .lines()
        .map(|l| Line::from(l.to_string()))
        .collect();
    f.render_widget(
        Paragraph::new(story)
            .block(Block::default().borders(Borders::ALL).title(title))
            .wrap(Wrap { trim: false }),
        body[0],
    );
    f.render_widget(side_panel(frame), body[1]);

    let footer = frame
        .slots
        .get("footer")
        .and_then(Value::as_str)
        .unwrap_or("q quit");
    f.render_widget(
        Paragraph::new(footer).style(
            Style::default()
                .bg(Color::Rgb(80, 80, 128))
                .fg(Color::Rgb(224, 224, 255)),
        ),
        chunks[2],
    );
    f.render_widget(Paragraph::new(status.to_string()), chunks[3]);
}

fn hud_line(frame: &Frame) -> String {
    let hud = &frame.slots["hud"];
    format!(
        " {}   {}   {}   {} ",
        hud.get("heir").and_then(Value::as_str).unwrap_or(""),
        hud.get("location").and_then(Value::as_str).unwrap_or(""),
        hud.get("phase").and_then(Value::as_str).unwrap_or(""),
        hud.get("money").and_then(Value::as_str).unwrap_or(""),
    )
}

fn narrative(frame: &Frame) -> &str {
    frame
        .slots
        .get("narrative")
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn side_panel(frame: &Frame) -> Paragraph<'static> {
    let sel = selected(frame);
    let mut lines: Vec<Line> = Vec::new();
    if let Some(list) = frame.slots.get("list").and_then(Value::as_array) {
        for (i, it) in list.iter().enumerate() {
            let name = it
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let mark = if i == sel { ">" } else { " " };
            let style = if i == sel {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            lines.push(Line::from(format!("{mark} {}. {name}", i + 1)).style(style));
        }
    }
    if lines.is_empty() {
        lines.push(Line::from("Enter continue"));
    }
    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" choix "))
}
