mod config;
mod info;
mod uwufy;
mod display;
mod cache;

use clap::Parser;
use std::io;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(name = "uwufetch")]
#[command(version = VERSION)]
#[command(about = "A system information fetcher with uwu", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config", help = "Use custom config path")]
    config: Option<String>,
    
    #[arg(short = 'd', long = "distro", help = "Choose the logo to print")]
    distro: Option<String>,
    
    #[arg(short = 'i', long = "image", help = "Print logo as image (requires viu)")]
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
    
    let mut user_info = if args.read_cache {
        if let Some(cached_info) = cache::read_cache() {
            cached_info
        } else {
            info::SystemInfo::new()
        }
    } else {
        info::SystemInfo::new()
    };
    
    if let Some(distro) = args.distro {
        user_info.os_name = distro;
    }
    
    let has_image = args.image.is_some();
    if let Some(image) = args.image {
        if !image.is_empty() {
            user_info.image_name = Some(image);
        }
    }
    
    let mut config = config::Configuration::parse_config(&mut user_info);
    
    if has_image {
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