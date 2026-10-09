use std::{
    env,
    io::{self, IsTerminal},
};

fn main() {
    // This binary will become the application that orchestrates provider calls
    // and tools. Bootstrap only prepares its safe local configuration.
    // A missing `.env` is normal during bootstrap; dotenvy leaves the process
    // environment unchanged in that case.
    dotenvy::dotenv().ok();

    print_status("Hello from gdk_hello");
}

// Presentation support only; redirected transcripts remain plain text.
#[derive(Clone, Copy)]
enum TextStyle {
    Green,
}

fn color_enabled(is_terminal: bool, no_color: bool, dumb_terminal: bool) -> bool {
    is_terminal && !no_color && !dumb_terminal
}

fn terminal_color_enabled(is_terminal: bool) -> bool {
    let no_color = env::var_os("NO_COLOR").is_some();
    let dumb_terminal = env::var_os("TERM") == Some("dumb".into());
    color_enabled(is_terminal, no_color, dumb_terminal)
}

fn styled(text: &str, style: TextStyle, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = match style {
        TextStyle::Green => "32",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

fn print_status(text: &str) {
    let enabled = terminal_color_enabled(io::stdout().is_terminal());
    println!("{}", styled(text, TextStyle::Green, enabled));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_respect_terminal_no_color_and_dumb_gates() {
        for is_terminal in [false, true] {
            for no_color in [false, true] {
                for dumb_terminal in [false, true] {
                    let expected = is_terminal && !no_color && !dumb_terminal;
                    assert_eq!(
                        color_enabled(is_terminal, no_color, dumb_terminal),
                        expected
                    );
                }
            }
        }
    }

    #[test]
    fn greeting_is_green_or_exactly_plain() {
        assert_eq!(
            styled("Hello from gdk_hello", TextStyle::Green, true),
            "\x1b[32mHello from gdk_hello\x1b[0m"
        );
        assert_eq!(
            styled("Hello from gdk_hello", TextStyle::Green, false),
            "Hello from gdk_hello"
        );
    }
}
