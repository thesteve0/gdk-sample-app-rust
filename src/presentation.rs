// Lesson 14
use std::{
    env,
    io::{self, IsTerminal},
};
// Terminal styling is presentation only; redirected transcripts contain no ANSI codes.
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

// Supplied streaming presentation helper: hide common prose markup even across deltas.
// This is not a general Markdown renderer; saved/provider text remains untouched.
pub(super) struct TerminalProse {
    line_start: bool,
}

impl Default for TerminalProse {
    fn default() -> Self {
        Self { line_start: true }
    }
}

impl TerminalProse {
    pub(super) fn fragment(&mut self, text: &str) -> String {
        let mut displayed = String::new();
        for character in text.chars() {
            if character == '*' || character == '`' || (self.line_start && character == '#') {
                continue;
            }
            if self.line_start && character == ' ' {
                continue;
            }
            if character == '|' {
                displayed.push_str(" / ");
            } else if !character.is_control() || character == '\n' || character == '\t' {
                displayed.push(character);
            }
            self.line_start = character == '\n';
        }
        displayed
    }
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

    #[test]
    fn live_prose_hides_common_markup_across_stream_fragments() {
        let mut prose = TerminalProse::default();
        let original = ["##", "# Heading\nFor **$", "100.00** and `orders`.\n"];
        let mut output = String::new();
        for fragment in original {
            output.push_str(&prose.fragment(fragment));
        }
        assert_eq!(output, "Heading\nFor $100.00 and orders.\n");
        assert!(original[1].contains("**"));
    }
}
