use crate::assets::Assets;
use crate::config::Configuration;
use crate::info::SystemInfo;
use crate::uwufy;
use owo_colors::{AnsiColors, OwoColorize, Rgb, Style};
use std::io::{self, BufWriter, Write};

const BLOCK_CHAR: &str = "█";

const TOK_NORMAL: &str = "NORMAL";
const TOK_BOLD: &str = "BOLD";
const TOK_BLACK: &str = "BLACK";
const TOK_RED: &str = "RED";
const TOK_GREEN: &str = "GREEN";
const TOK_SPRING_GREEN: &str = "SPRING_GREEN";
const TOK_YELLOW: &str = "YELLOW";
const TOK_BLUE: &str = "BLUE";
const TOK_MAGENTA: &str = "MAGENTA";
const TOK_CYAN: &str = "CYAN";
const TOK_WHITE: &str = "WHITE";
const TOK_PINK: &str = "PINK";
const TOK_LPINK: &str = "LPINK";
const TOK_BLOCK: &str = "BLOCK";
const TOK_BLOCK_VERT: &str = "BLOCK_VERTICAL";
const TOK_BG_GREEN: &str = "BACKGROUND_GREEN";
const TOK_BG_RED: &str = "BACKGROUND_RED";
const TOK_BG_WHITE: &str = "BACKGROUND_WHITE";

#[derive(Clone, Copy)]
enum ColorSpec {
    Ansi(AnsiColors),
    Rgb(u8, u8, u8),
}

#[derive(Default, Clone, Copy)]
struct StyleState {
    bold: bool,
    fg: Option<ColorSpec>,
    bg: Option<ColorSpec>,
}

fn apply_style(s: &str, st: StyleState) -> String {
    if s.is_empty() {
        return String::new();
    }
    let mut style = Style::new();
    if st.bold {
        style = style.bold();
    }
    if let Some(fg) = st.fg {
        style = match fg {
            ColorSpec::Ansi(c) => style.color(c),
            ColorSpec::Rgb(r, g, b) => style.color(Rgb(r, g, b)),
        };
    }
    if let Some(bg) = st.bg {
        style = match bg {
            ColorSpec::Ansi(c) => style.on_color(c),
            ColorSpec::Rgb(r, g, b) => style.on_color(Rgb(r, g, b)),
        };
    }
    format!("{}", s.style(style))
}

fn render_ascii(content: &str) -> String {
    let mut out = String::new();
    let mut st = StyleState::default();
    let mut rest = content;

    while let Some(start) = rest.find('{') {
        let before = &rest[..start];
        out.push_str(&apply_style(before, st));

        let after_brace = &rest[start + 1..];
        if let Some(end_rel) = after_brace.find('}') {
            let token = &after_brace[..end_rel];
            rest = &after_brace[end_rel + 1..];

            match token {
                TOK_NORMAL => {
                    st = StyleState::default();
                }
                TOK_BOLD => {
                    st.bold = true;
                }
                TOK_BLACK => st.fg = Some(ColorSpec::Ansi(AnsiColors::Black)),
                TOK_RED => st.fg = Some(ColorSpec::Ansi(AnsiColors::Red)),
                TOK_GREEN => st.fg = Some(ColorSpec::Ansi(AnsiColors::Green)),
                TOK_SPRING_GREEN => {
                    st.fg = Some(ColorSpec::Rgb(0, 255, 127));
                }
                TOK_YELLOW => st.fg = Some(ColorSpec::Ansi(AnsiColors::Yellow)),
                TOK_BLUE => st.fg = Some(ColorSpec::Ansi(AnsiColors::Blue)),
                TOK_MAGENTA => st.fg = Some(ColorSpec::Ansi(AnsiColors::Magenta)),
                TOK_CYAN => st.fg = Some(ColorSpec::Ansi(AnsiColors::Cyan)),
                TOK_WHITE => st.fg = Some(ColorSpec::Ansi(AnsiColors::White)),
                TOK_PINK => {
                    st.fg = Some(ColorSpec::Rgb(255, 105, 180));
                }
                TOK_LPINK => {
                    st.fg = Some(ColorSpec::Rgb(255, 182, 193));
                }
                TOK_BG_GREEN => {
                    st.bg = Some(ColorSpec::Ansi(AnsiColors::Green));
                }
                TOK_BG_RED => {
                    st.bg = Some(ColorSpec::Ansi(AnsiColors::Red));
                }
                TOK_BG_WHITE => {
                    st.bg = Some(ColorSpec::Ansi(AnsiColors::White));
                }
                TOK_BLOCK | TOK_BLOCK_VERT => {
                    out.push_str(&apply_style(BLOCK_CHAR, st));
                }
                _ => {
                    out.push('{');
                    out.push_str(token);
                    out.push('}');
                }
            }
        } else {
            out.push_str(&apply_style(&rest[start..], st));
            return out;
        }
    }

    out.push_str(&apply_style(rest, st));
    out
}

#[allow(clippy::write_literal)]
pub fn print_info(config: &Configuration, info: &mut SystemInfo) -> io::Result<()> {
    let mut out = BufWriter::new(io::stdout());

    uwufy::uwu_name(&mut info.os_name);

    let move_cursor = "\x1b[18C";

    if config.show_user {
        let userhost = format!("{}@{}", info.user, info.host);
        writeln!(&mut out, "{}{}", move_cursor, userhost.bold())?;
    }

    if config.show_os {
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "OWOS     ".bold(),
            info.os_name
        )?;
    }

    if config.show_host {
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "MOWODEL  ".bold(),
            info.model
        )?;
    }

    if config.show_kernel {
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "KEWNEL   ".bold(),
            info.kernel
        )?;
    }

    if config.show_cpu {
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "CPUWU    ".bold(),
            info.cpu_model
        )?;
    }

    if config.show_gpu {
        for gpu in &info.gpu_models {
            writeln!(&mut out, "{}{} {}", move_cursor, "GPUWU    ".bold(), gpu)?;
        }
    }

    if config.show_ram {
        writeln!(
            &mut out,
            "{}{} {} MiB/{} MiB",
            move_cursor,
            "MEMOWY   ".bold(),
            info.ram_used,
            info.ram_total
        )?;
    }

    if config.show_resolution && (info.screen_width != 0 || info.screen_height != 0) {
        writeln!(
            &mut out,
            "{}{} {}x{}",
            move_cursor,
            "WESOWUTION".bold(),
            info.screen_width,
            info.screen_height
        )?;
    }

    if config.show_shell {
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "SHEWW    ".bold(),
            info.shell
        )?;
    }

    if config.show_pkgs {
        writeln!(
            &mut out,
            "{}{} {}: {}",
            move_cursor,
            "PKGS     ".bold(),
            info.pkgs,
            info.pkgman_name
        )?;
    }

    if config.show_uptime {
        let uptime_str = format_uptime(info.uptime);
        writeln!(
            &mut out,
            "{}{} {}",
            move_cursor,
            "UWUPTIME ".bold(),
            uptime_str
        )?;
    }

    if config.show_colors {
        writeln!(
            &mut out,
            "{}{}{}{}{}{}{}{}",
            "\x1b[18C",
            "██".black(),
            "██".red(),
            "██".green(),
            "██".yellow(),
            "██".blue(),
            "██".magenta(),
            "██".cyan()
        )?;
        writeln!(&mut out, "{}{}", "\x1b[18C", "██".white())?;
    }

    out.flush()?;
    Ok(())
}

fn format_uptime(seconds: u64) -> String {
    match seconds {
        0..=3599 => format!("{}m", seconds / 60 % 60),
        3600..=86399 => format!("{}h, {}m", seconds / 3600, seconds / 60 % 60),
        _ => format!(
            "{}d, {}h, {}m",
            seconds / 86400,
            seconds / 3600 % 24,
            seconds / 60 % 60
        ),
    }
}

pub fn print_ascii(info: &SystemInfo) -> io::Result<usize> {
    let mut out = BufWriter::new(io::stdout());
    let ascii_filename = format!("ascii/{}.txt", info.os_name);

    if let Some(file) = Assets::get(&ascii_filename) {
        let content = std::str::from_utf8(&file.data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let processed = render_ascii(content);
        writeln!(&mut out)?;
        out.write_all(processed.as_bytes())?;
        out.flush()?;

        let line_count = processed.lines().count() + 1;

        return Ok(line_count);
    }

    if info.os_name != "unknown" {
        let fallback_filename = "ascii/unknown.txt";
        if let Some(file) = Assets::get(fallback_filename) {
            let content = std::str::from_utf8(&file.data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let processed = render_ascii(content);
            writeln!(&mut out)?;
            out.write_all(processed.as_bytes())?;
            out.flush()?;

            let line_count = processed.lines().count() + 1;
            return Ok(line_count);
        }
    }

    writeln!(&mut out, "No\nascii\nfile\nfound\n\n\n")?;
    out.flush()?;

    Ok(7)
}

pub fn print_image(info: &SystemInfo) -> io::Result<usize> {
    let image_filename = if let Some(ref img) = info.image_name {
        format!("{}.sixel", img)
    } else {
        format!("{}.sixel", info.os_name)
    };

    if let Some(file) = Assets::get(&image_filename) {
        let sixelstr = std::str::from_utf8(&file.data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        println!("{}", sixelstr);
        return Ok(9);
    }

    if info.image_name.is_none() && info.os_name != "unknown" {
        if let Some(file) = Assets::get("unknown.sixel") {
            let sixelstr = std::str::from_utf8(&file.data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            println!("{}", sixelstr);
            return Ok(9);
        }
    }

    println!("No image found");
    Ok(1)
}
