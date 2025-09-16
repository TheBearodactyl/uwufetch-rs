use crate::config::Configuration;
use crate::info::SystemInfo;
use crate::uwufy;
use aho_corasick::AhoCorasick;
use once_cell::sync::Lazy;
use std::fs::{self};
use std::io::{self, BufWriter, Write};
use std::path::Path;

const BLOCK_CHAR: &str = "█";

static ASCII_PLACEHOLDERS: [&str; 18] = [
    "{NORMAL}",
    "{BOLD}",
    "{BLACK}",
    "{RED}",
    "{GREEN}",
    "{SPRING_GREEN}",
    "{YELLOW}",
    "{BLUE}",
    "{MAGENTA}",
    "{CYAN}",
    "{WHITE}",
    "{PINK}",
    "{LPINK}",
    "{BLOCK}",
    "{BLOCK_VERTICAL}",
    "{BACKGROUND_GREEN}",
    "{BACKGROUND_RED}",
    "{BACKGROUND_WHITE}",
];

static ASCII_REPLACEMENTS: [&str; 18] = [
    "\x1b[0m",
    "\x1b[1m",
    "\x1b[30m",
    "\x1b[31m",
    "\x1b[32m",
    "\x1b[38;5;120m",
    "\x1b[33m",
    "\x1b[34m",
    "\x1b[0;35m",
    "\x1b[36m",
    "\x1b[37m",
    "\x1b[38;5;201m",
    "\x1b[38;5;213m",
    BLOCK_CHAR,
    BLOCK_CHAR,
    "\x1b[0;42m",
    "\x1b[0;41m",
    "\x1b[0;47m",
];

static ASCII_AC: Lazy<AhoCorasick> = Lazy::new(|| {
    AhoCorasick::new(ASCII_PLACEHOLDERS).expect("Failed to build Aho-Corasick automaton")
});

pub fn print_info(config: &Configuration, info: &mut SystemInfo) -> io::Result<()> {
    let mut out = BufWriter::new(std::io::stdout());

    uwufy::uwu_name(&mut info.os_name);

    let move_cursor = "\x1b[18C";

    if config.show_user {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1m{}@{}",
            move_cursor, info.user, info.host
        )?;
    }

    if config.show_os {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mOWOS     \x1b[0m{}",
            move_cursor, info.os_name
        )?;
    }

    if config.show_host {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mMOWODEL  \x1b[0m{}",
            move_cursor, info.model
        )?;
    }

    if config.show_kernel {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mKEWNEL   \x1b[0m{}",
            move_cursor, info.kernel
        )?;
    }

    if config.show_cpu {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mCPUWU    \x1b[0m{}",
            move_cursor, info.cpu_model
        )?;
    }

    if config.show_gpu {
        for gpu in &info.gpu_models {
            writeln!(
                &mut out,
                "{}\x1b[0m\x1b[1mGPUWU    \x1b[0m{}",
                move_cursor, gpu
            )?;
        }
    }

    if config.show_ram {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mMEMOWY   \x1b[0m{} MiB/{} MiB",
            move_cursor, info.ram_used, info.ram_total
        )?;
    }

    if config.show_resolution && (info.screen_width != 0 || info.screen_height != 0) {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mWESOWUTION\x1b[0m  {}x{}",
            move_cursor, info.screen_width, info.screen_height
        )?;
    }

    if config.show_shell {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mSHEWW    \x1b[0m{}",
            move_cursor, info.shell
        )?;
    }

    if config.show_pkgs {
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mPKGS     \x1b[0m{}: {}",
            move_cursor, info.pkgs, info.pkgman_name
        )?;
    }

    if config.show_uptime {
        let uptime_str = format_uptime(info.uptime);
        writeln!(
            &mut out,
            "{}\x1b[0m\x1b[1mUWUPTIME \x1b[0m{}",
            move_cursor, uptime_str
        )?;
    }

    if config.show_colors {
        writeln!(
            &mut out,
            "\x1b[18C\x1b[30m██\x1b[31m██\x1b[32m██\x1b[33m██\
             \x1b[34m██\x1b[35m██\x1b[36m██\x1b[37m██\x1b[0m"
        )?;
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
    let ascii_path = find_ascii_file(&info.os_name);

    let mut out = BufWriter::new(std::io::stdout());

    if let Some(path) = ascii_path {
        if let Ok(content) = fs::read_to_string(&path) {
            let processed = ASCII_AC.replace_all(&content, &ASCII_REPLACEMENTS);
            writeln!(&mut out)?;
            out.write_all(processed.as_bytes())?;
            write!(&mut out, "\x1b[0m")?;
            out.flush()?;

            let line_count = processed.lines().count() + 1;
            return Ok(line_count);
        }
    }

    writeln!(&mut out, "No\nascii\nfile\nfound\n\n\n")?;
    out.flush()?;
    Ok(7)
}

fn find_ascii_file(os_name: &str) -> Option<String> {
    let local_path = format!("./res/ascii/{}.txt", os_name);
    if Path::new(&local_path).exists() {
        return Some(local_path);
    }

    #[cfg(target_os = "android")]
    let system_path = format!(
        "/data/data/com.termux/files/usr/lib/uwufetch/ascii/{}.txt",
        os_name
    );

    #[cfg(target_os = "macos")]
    let system_path = format!("/usr/local/lib/uwufetch/ascii/{}.txt", os_name);

    #[cfg(not(any(target_os = "android", target_os = "macos")))]
    let system_path = format!("/usr/lib/uwufetch/ascii/{}.txt", os_name);

    if Path::new(&system_path).exists() {
        Some(system_path)
    } else if os_name != "unknown" {
        find_ascii_file("unknown")
    } else {
        None
    }
}

pub fn print_image(info: &SystemInfo) -> io::Result<usize> {
    let image_path = if let Some(ref img) = info.image_name {
        let local = format!("./res/{}.sixel", img);
        if Path::new(&local).exists() {
            local
        } else {
            img.clone()
        }
    } else {
        #[cfg(target_os = "android")]
        let default_path = format!(
            "/data/data/com.termux/files/usr/lib/uwufetch/{}.sixel",
            info.os_name
        );

        #[cfg(target_os = "macos")]
        let default_path = format!("/usr/local/lib/uwufetch/{}.sixel", info.os_name);

        #[cfg(not(any(target_os = "android", target_os = "macos")))]
        let default_path = format!("/usr/lib/uwufetch/{}.sixel", info.os_name);

        let local = format!("./res/{}.sixel", info.os_name);
        if Path::new(&local).exists() {
            local
        } else {
            default_path
        }
    };

    let sixelstr = fs::read_to_string(image_path)?;
    println!("{}", sixelstr);
    Ok(9)
}
