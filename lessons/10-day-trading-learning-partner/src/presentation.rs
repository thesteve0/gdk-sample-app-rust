// Lesson 10
use std::{
    env,
    io::{self, IsTerminal, Write},
};
const PAYLOAD_DELIMITER: &str = "++++++++";
#[derive(Clone, Copy)]
pub(super) enum TextStyle {
    Heading,
    Blue,
    Magenta,
    Green,
    Yellow,
    Error,
}

fn color_enabled(is_terminal: bool, no_color: bool, dumb_terminal: bool) -> bool {
    is_terminal && !no_color && !dumb_terminal
}

fn terminal_color_enabled(is_terminal: bool) -> bool {
    let no_color = env::var_os("NO_COLOR").is_some();
    let dumb_terminal = env::var_os("TERM") == Some("dumb".into());
    color_enabled(is_terminal, no_color, dumb_terminal)
}

pub(super) fn stdout_color_enabled() -> bool {
    terminal_color_enabled(io::stdout().is_terminal())
}

pub(super) fn styled(text: &str, style: TextStyle, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = match style {
        TextStyle::Heading => "1;36",
        TextStyle::Blue => "34",
        TextStyle::Magenta => "35",
        TextStyle::Green => "32",
        TextStyle::Yellow => "33",
        TextStyle::Error => "1;31",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

pub(super) fn print_heading(text: &str) {
    println!(
        "{}",
        styled(text, TextStyle::Heading, stdout_color_enabled())
    );
}

pub(super) fn print_status(text: &str) {
    println!("{}", styled(text, TextStyle::Green, stdout_color_enabled()));
}

pub(super) fn print_warning(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Yellow, enabled));
}

pub(super) fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

pub(super) fn print_delimiter<W: Write>(output: &mut W) -> io::Result<()> {
    writeln!(output, "{}", PAYLOAD_DELIMITER)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn terminal_colors_respect_redirection_no_color_and_dumb_terminals() {
        assert!(color_enabled(true, false, false));
        assert!(!color_enabled(false, false, false));
        assert!(!color_enabled(true, true, false));
        assert!(!color_enabled(true, false, true));
        assert_eq!(styled("payload", TextStyle::Blue, false), "payload");
        assert_eq!(
            styled("payload", TextStyle::Blue, true),
            "\x1b[34mpayload\x1b[0m"
        );
    }
}
