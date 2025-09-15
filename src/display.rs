use crate::config::Configuration;
use crate::info::SystemInfo;
use crate::uwufy;
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use crossterm::{cursor, ExecutableCommand};
use std::fs;
use std::io::stdout;
use std::io::{self};
use std::path::Path;
use std::process::Command;

const BLOCK_CHAR: &str = "█";

pub fn print_info(config: &Configuration, info: &mut SystemInfo) -> io::Result<()> {
    let mut stdout = stdout();

    uwufy::uwu_name(&mut info.os_name);

    let move_cursor = "\x1b[18C";

    if config.show_user {
        println!("{}\x1b[0m\x1b[1m{}@{}", move_cursor, info.user, info.host);
    }

    if config.show_os {
        println!(
            "{}\x1b[0m\x1b[1mOWOS     \x1b[0m{}",
            move_cursor, info.os_name
        );
    }

    if config.show_host {
        println!(
            "{}\x1b[0m\x1b[1mMOWODEL  \x1b[0m{}",
            move_cursor, info.model
        );
    }

    if config.show_kernel {
        println!(
            "{}\x1b[0m\x1b[1mKEWNEL   \x1b[0m{}",
            move_cursor, info.kernel
        );
    }

    if config.show_cpu {
        println!(
            "{}\x1b[0m\x1b[1mCPUWU    \x1b[0m{}",
            move_cursor, info.cpu_model
        );
    }

    if config.show_gpu {
        for gpu in &info.gpu_models {
            println!("{}\x1b[0m\x1b[1mGPUWU    \x1b[0m{}", move_cursor, gpu);
        }
    }

    if config.show_ram {
        println!(
            "{}\x1b[0m\x1b[1mMEMOWY   \x1b[0m{} MiB/{} MiB",
            move_cursor, info.ram_used, info.ram_total
        );
    }

    if config.show_resolution && (info.screen_width != 0 || info.screen_height != 0) {
        println!(
            "{}\x1b[0m\x1b[1mWESOWUTION\x1b[0m  {}x{}",
            move_cursor, info.screen_width, info.screen_height
        );
    }

    if config.show_shell {
        println!(
            "{}\x1b[0m\x1b[1mSHEWW    \x1b[0m{}",
            move_cursor, info.shell
        );
    }

    if config.show_pkgs {
        println!(
            "{}\x1b[0m\x1b[1mPKGS     \x1b[0m{}: {}",
            move_cursor, info.pkgs, info.pkgman_name
        );
    }

    if config.show_uptime {
        let uptime_str = format_uptime(info.uptime);
        println!(
            "{}\x1b[0m\x1b[1mUWUPTIME \x1b[0m{}",
            move_cursor, uptime_str
        );
    }

    if config.show_colors {
        stdout.execute(cursor::MoveToColumn(18))?;
        stdout.execute(SetForegroundColor(Color::Black))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Red))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Green))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Yellow))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Blue))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Magenta))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::Cyan))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(SetForegroundColor(Color::White))?;
        print!("{}{}", BLOCK_CHAR, BLOCK_CHAR);
        stdout.execute(ResetColor)?;
        println!();
    }

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

    if let Some(path) = ascii_path {
        if let Ok(content) = fs::read_to_string(&path) {
            let mut line_count = 1;
            println!();

            for line in content.lines() {
                let processed = process_ascii_line(line);
                println!("{}", processed);
                line_count += 1;
            }

            print!("\x1b[0m");
            return Ok(line_count);
        }
    }

    println!("No\nascii\nfile\nfound\n\n\n");
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

fn process_ascii_line(line: &str) -> String {
    let mut result = line.to_string();

    let replacements = [
        ("{NORMAL}", "\x1b[0m"),
        ("{BOLD}", "\x1b[1m"),
        ("{BLACK}", "\x1b[30m"),
        ("{RED}", "\x1b[31m"),
        ("{GREEN}", "\x1b[32m"),
        ("{SPRING_GREEN}", "\x1b[38;5;120m"),
        ("{YELLOW}", "\x1b[33m"),
        ("{BLUE}", "\x1b[34m"),
        ("{MAGENTA}", "\x1b[0;35m"),
        ("{CYAN}", "\x1b[36m"),
        ("{WHITE}", "\x1b[37m"),
        ("{PINK}", "\x1b[38;5;201m"),
        ("{LPINK}", "\x1b[38;5;213m"),
        ("{BLOCK}", BLOCK_CHAR),
        ("{BLOCK_VERTICAL}", BLOCK_CHAR),
        ("{BACKGROUND_GREEN}", "\x1b[0;42m"),
        ("{BACKGROUND_RED}", "\x1b[0;41m"),
        ("{BACKGROUND_WHITE}", "\x1b[0;47m"),
    ];

    for (placeholder, replacement) in &replacements {
        result = result.replace(placeholder, replacement);
    }

    result
}

pub fn print_image(info: &SystemInfo) -> io::Result<usize> {
    let image_path = if let Some(ref custom_image) = info.image_name {
        custom_image.clone()
    } else {
        #[cfg(target_os = "android")]
        let default_path = format!(
            "/data/data/com.termux/files/usr/lib/uwufetch/{}.png",
            info.os_name
        );

        #[cfg(target_os = "macos")]
        let default_path = format!("/usr/local/lib/uwufetch/{}.png", info.os_name);

        #[cfg(not(any(target_os = "android", target_os = "macos")))]
        let default_path = format!("/usr/lib/uwufetch/{}.png", info.os_name);

        default_path
    };

    let output = Command::new("viu")
        .args(["-t", "-w", "18", "-h", "9", &image_path])
        .output();

    if output.is_err() || !output.unwrap().status.success() {
        println!("\x1b[0E\x1b[3C\x1b[31m");
        println!("   There was an");
        println!("    error: viu");
        println!("  is not installed");
        println!(" or the image file");
        println!("   was not found");
        println!("   see IMAGES.md");
        println!("   for more info.\n");
    }

    Ok(9)
}

