use super::diagram_formatting::FormattedEntry;
use lscolors::{self, LsColors, Style};
use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

pub fn print_entries(entries: &[FormattedEntry], create_alias: bool, lscolors: Option<&LsColors>) {
    let stdout = BufferWriter::stdout(ColorChoice::Auto);
    let mut buffer = stdout.buffer();
    let number_color = lscolors.map(|_| ColorSpec::new().set_fg(Some(Color::Red)).to_owned());
    
    for (index, entry) in entries.iter().enumerate() {
        if create_alias {
            let _ = write!(&mut buffer, "{}[", entry.prefix);
            if let Some(ref color_spec) = number_color {
                let _ = buffer.set_color(color_spec);
            }
            let _ = write!(&mut buffer, "{}", index);
            if number_color.is_some() {
                let _ = buffer.reset();
            }
            let _ = write!(&mut buffer, "] ");
        } else {
            let _ = write!(&mut buffer, "{}", entry.prefix);
        }

        if let Some(lsc) = lscolors {
            let spec = lsc.style_for_path(&entry.path)
                .map(|s| convert_to_color_spec(s));
            if let Some(ref color_spec) = spec {
                let _ = buffer.set_color(color_spec);
            }
            let _ = write!(&mut buffer, "{}", entry.name);
            if spec.is_some() {
                let _ = buffer.reset();
            }
        } else {
            let _ = write!(&mut buffer, "{}", entry.name);
        }
        let _ = writeln!(&mut buffer);
    }
    let _ = stdout.print(&buffer);
}

fn convert_color(color: &lscolors::Color) -> Color {
    match color {
        lscolors::Color::RGB(r, g, b) => Color::Rgb(*r, *g, *b),
        lscolors::Color::Fixed(n) => Color::Ansi256(*n),
        lscolors::Color::Black => Color::Black,
        lscolors::Color::Red => Color::Red,
        lscolors::Color::Green => Color::Green,
        lscolors::Color::Yellow => Color::Yellow,
        lscolors::Color::Blue => Color::Blue,
        lscolors::Color::Magenta => Color::Magenta,
        lscolors::Color::Cyan => Color::Cyan,
        lscolors::Color::White => Color::White,

        // bright colors don't have a direct alternative in termcolor::color
        // translate them to "ansi 256" colors using the same value as used
        // in to_ansi_term_color
        lscolors::Color::BrightBlack => Color::Ansi256(8),
        lscolors::Color::BrightRed => Color::Ansi256(9),
        lscolors::Color::BrightGreen => Color::Ansi256(10),
        lscolors::Color::BrightYellow => Color::Ansi256(11),
        lscolors::Color::BrightBlue => Color::Ansi256(12),
        lscolors::Color::BrightMagenta => Color::Ansi256(13),
        lscolors::Color::BrightCyan => Color::Ansi256(14),
        lscolors::Color::BrightWhite => Color::Ansi256(15),
    }
}

fn convert_to_color_spec(style: &Style) -> ColorSpec {
    let mut spec = ColorSpec::new();

    if let Some(color) = &style.foreground {
        spec.set_fg(Some(convert_color(color)));
    }

    if let Some(color) = &style.background {
        spec.set_bg(Some(convert_color(color)));
    }

    spec.set_bold(style.font_style.bold);
    spec.set_italic(style.font_style.italic);
    spec.set_underline(style.font_style.underline);

    spec
}

#[cfg(target_os = "windows")]
fn open_alias_file_with_suffix(suffix: &str) -> io::Result<File> {
    let file_name = format!(
        "tre_aliases_{}.{}",
        env::var("USERNAME").unwrap_or_else(|_| "".to_string()),
        suffix
    );
    let home = env::var("HOME").unwrap_or_else(|_| r".".to_string());
    let tmp = env::var("TEMP").unwrap_or(home);
    let path: PathBuf = [tmp, file_name].iter().collect();
    let file = File::create(&path);
    if file.is_err() {
        eprintln!("[tre] failed to open {:?}", path);
    }

    file
}

#[cfg(target_os = "windows")]
pub fn create_edit_aliases(editor: &str, entries: &[FormattedEntry]) {
    let powershell_alias = open_alias_file_with_suffix("ps1");
    if let Ok(mut alias_file) = powershell_alias {
        for (index, entry) in entries.iter().enumerate() {
            let editor = if editor.is_empty() {
                "Start-Process"
            } else {
                editor
            };
            let result = writeln!(
                &mut alias_file,
                "doskey /exename=pwsh.exe e{}={} {}\ndoskey /exename=powershell.exe e{}={} {}",
                index, editor, entry.path, index, editor, entry.path,
            );

            if result.is_err() {
                eprintln!("[tre] failed to write to alias file.");
            }
        }
    }

    let cmd_alias = open_alias_file_with_suffix("bat");
    if let Ok(mut alias_file) = cmd_alias {
        for (index, entry) in entries.iter().enumerate() {
            let editor = if editor.is_empty() { "START" } else { editor };
            let result = writeln!(
                &mut alias_file,
                "doskey /exename=cmd.exe e{}={} {}",
                index, editor, entry.path,
            );

            if result.is_err() {
                eprintln!("[tre] failed to write to alias file.");
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn open_alias_file() -> io::Result<File> {
    let user = env::var("USER").unwrap_or_else(|_| "".to_string());
    let alias_file = format!("/tmp/tre_aliases_{}", &user);
    let path: PathBuf = [alias_file].iter().collect();
    let file = File::create(&path);
    if file.is_err() {
        eprintln!("[tre] failed to open {:?}", path);
    }

    file
}

#[cfg(not(target_os = "windows"))]
pub fn create_edit_aliases(editor: &str, entries: &[FormattedEntry]) {
    let alias = open_alias_file();
    if let Ok(mut alias_file) = alias {
        for (index, entry) in entries.iter().enumerate() {
            let result = writeln!(
                &mut alias_file,
                "alias e{}=\"eval '{} \\\"{}\\\"'\"",
                index,
                editor,
                entry.path.replace('\'', "\\'")
            );

            if result.is_err() {
                eprintln!("[tre] failed to write to alias file.");
            }
        }
    }
}
