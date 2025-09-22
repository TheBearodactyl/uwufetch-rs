mod assets;
mod cache;
mod config;
mod display;
mod info;
mod uwufy;

use clap::Parser;
use std::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "uwufetch")]
#[command(version = VERSION)]
#[command(about = "A system information fetcher with uwu", long_about = None)]
#[command(disable_version_flag = true)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Use custom config path")]
    config: Option<String>,

    #[arg(short = 'd', long = "distro", help = "Choose the logo to print")]
    distro: Option<String>,

    #[arg(
        short = 'i',
        long = "image",
        help = "Print logo as image (requires sixel)"
    )]
    image: Option<String>,

    #[arg(short = 'l', long = "list", help = "List all supported distributions")]
    list: bool,

    #[arg(short = 'r', long = "read-cache", help = "Read from cache file")]
    read_cache: bool,

    #[arg(short = 'w', long = "write-cache", help = "Write to cache file")]
    write_cache: bool,

    #[arg(short = 'V', long = "version", help = "Print version")]
    version: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.list {
        list_distributions();
        return Ok(());
    }

    if args.version {
        println!("UwUfetch version {}", VERSION);
        return Ok(());
    }

    let (mut config, distro_override, image_override) = config::Configuration::parse_config();
    let mut user_info_opt = if args.read_cache {
        cache::read_cache()
    } else {
        None
    };

    let cli_distro = args.distro;
    let cli_image = args.image;

    let mut user_info = if let Some(mut info) = user_info_opt.take() {
        if let Some(d) = cli_distro.clone().or_else(|| distro_override.clone()) {
            info.os_name = d;
        }
        if let Some(img) = cli_image.clone().or_else(|| image_override.clone()) {
            if !img.is_empty() {
                info.image_name = Some(img);
                config.show_image = true;
            }
        }
        info
    } else {
        let mut info = info::SystemInfo::default();
        if let Some(d) = cli_distro.clone().or_else(|| distro_override.clone()) {
            info.os_name = d;
        }
        if let Some(img) = cli_image.clone().or_else(|| image_override.clone()) {
            if !img.is_empty() {
                info.image_name = Some(img);
                config.show_image = true;
            }
        }
        info.populate(&config);
        info
    };

    let has_image_cli = cli_image.is_some();
    if has_image_cli {
        config.show_image = true;
    }

    if args.write_cache {
        cache::write_cache(&user_info);
    }

    uwufy::uwufy_all(&mut user_info);

    let lines_printed = if config.show_image {
        display::print_image(&user_info)?
    } else {
        display::print_ascii(&user_info)?
    };

    print!("\x1b[{}A", lines_printed);

    display::print_info(&config, &mut user_info)?;

    let move_amount = 9i32 - lines_printed as i32;
    if move_amount < 0 {
        print!("\x1b[{}A", -move_amount);
    } else if move_amount > 0 {
        print!("\x1b[{}B", move_amount);
    }

    Ok(())
}

fn list_distributions() {
    println!("uwufetch -d <options>");
    println!("  Available distributions:");
    println!("    \x1b[34mArch linux \x1b[0mbased:");
    println!("      \x1b[34march, arcolinux, \x1b[35martix, endeavouros \x1b[32mmanjaro, manjaro-arm, \x1b[34mxerolinux\n");
    println!("    \x1b[31mDebian/\x1b[33mUbuntu \x1b[0mbased:");
    println!("      \x1b[31mamogos, debian, deepin, \x1b[32mlinuxmint, neon, \x1b[34mpop, \x1b[31mraspbian \x1b[33mubuntu\n");
    println!("    \x1b[31mBSD \x1b[0mbased:");
    println!("      \x1b[31mfreebsd, \x1b[33mopenbsd, \x1b[32mm\x1b[33ma\x1b[31mc\x1b[38;5;201mo\x1b[34ms, \x1b[37mios\n");
    println!("    \x1b[31mRHEL \x1b[0mbased:");
    println!("      \x1b[34mfedora, \x1b[32mrocky\n");
    println!("    \x1b[0mOther/spare distributions:");
    println!("      \x1b[34malpine, \x1b[38;5;201mfemboyos, gentoo, \x1b[35mslackware, \x1b[37msolus, \x1b[32mvoid, opensuse-leap, android, \x1b[33mgnu, guix, \x1b[34mwindows, \x1b[37munknown\n");
}
