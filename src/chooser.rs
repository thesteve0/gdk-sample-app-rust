// Lesson 13
use crate::{
    presentation::{print_heading, print_warning, stdout_color_enabled, styled, TextStyle},
    LESSON_TITLE,
};
use crossterm::{
    cursor::MoveUp,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::{
    env,
    io::{self, IsTerminal, Write},
};
pub(super) fn read_line(prompt: &str) -> io::Result<Option<String>> {
    print!(
        "{}",
        styled(prompt, TextStyle::Blue, stdout_color_enabled())
    );
    io::stdout().flush()?;
    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input)?;
    if bytes == 0 {
        return Ok(None);
    }
    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }
    Ok(Some(input))
}

// Raw mode is confined to the chooser and restored before any provider call/input line.
struct RawMode;

impl RawMode {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Best-effort restoration on an earlier error; success is checked explicitly below.
        let _restore = disable_raw_mode();
    }
}

pub(super) fn choose_lesson() -> anyhow::Result<bool> {
    if io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && env::var_os("TERM") != Some("dumb".into())
        && env::var_os("NO_COLOR").is_none()
    {
        return arrow_chooser();
    }
    print_heading("\nChoose a lesson:");
    println!("1. {}\nq. Quit", LESSON_TITLE);
    loop {
        let input = match read_line("Choice > ")? {
            Some(input) => input,
            None => return Ok(false),
        };
        match input.trim() {
            "1" => return Ok(true),
            "q" | "/quit" => return Ok(false),
            _ => print_warning("Choose 1 or q."),
        }
    }
}

fn arrow_chooser() -> anyhow::Result<bool> {
    print_heading("\nChoose a lesson (↑/↓, Enter; Esc or q exits):");
    let raw_mode = RawMode::enter()?;
    let mut selected = 0;
    let mut stdout = io::stdout();
    loop {
        let lesson_line = if selected == 0 { "> " } else { "  " };
        let quit_line = if selected == 1 { "> " } else { "  " };
        write!(
            stdout,
            "{}{}\r\n{}Quit\r\n",
            lesson_line,
            styled(LESSON_TITLE, TextStyle::Heading, stdout_color_enabled()),
            quit_line
        )?;
        stdout.flush()?;
        loop {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                let mut decision = None;
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        selected = 1 - selected;
                    }
                    KeyCode::Enter => decision = Some(selected == 0),
                    KeyCode::Esc | KeyCode::Char('q') => decision = Some(false),
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            decision = Some(false);
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                }
                if let Some(start_lesson) = decision {
                    disable_raw_mode()?;
                    drop(raw_mode);
                    return Ok(start_lesson);
                }
                execute!(stdout, MoveUp(2), Clear(ClearType::FromCursorDown))?;
                break;
            }
        }
    }
}
