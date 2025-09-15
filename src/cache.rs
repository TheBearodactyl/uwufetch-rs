use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use crate::info::SystemInfo;

pub fn write_cache(info: &SystemInfo) {
    if let Ok(home) = std::env::var("HOME") {
        let cache_dir = PathBuf::from(home).join(".cache");
        if !cache_dir.exists() {
            let _ = fs::create_dir(&cache_dir);
        }
        
        let cache_file = cache_dir.join("uwufetch.cache");
        
        if let Ok(mut file) = File::create(cache_file) {
            let _ = writeln!(file, "user={}", info.user);
            let _ = writeln!(file, "host={}", info.host);
            let _ = writeln!(file, "version_name={}", info.os_name);
            let _ = writeln!(file, "host_model={}", info.model);
            let _ = writeln!(file, "kernel={}", info.kernel);
            let _ = writeln!(file, "cpu={}", info.cpu_model);
            let _ = writeln!(file, "screen_width={}", info.screen_width);
            let _ = writeln!(file, "screen_height={}", info.screen_height);
            let _ = writeln!(file, "shell={}", info.shell);
            let _ = writeln!(file, "pkgs={}", info.pkgs);
            let _ = writeln!(file, "pkgman_name={}", info.pkgman_name);
            
            for gpu in &info.gpu_models {
                let _ = writeln!(file, "gpu={}", gpu);
            }
        }
    }
}

pub fn read_cache() -> Option<SystemInfo> {
    if let Ok(home) = std::env::var("HOME") {
        let cache_file = PathBuf::from(home).join(".cache/uwufetch.cache");
        
        if let Ok(file) = File::open(cache_file) {
            let reader = BufReader::new(file);
            let mut info = SystemInfo {
                user: String::new(),
                host: String::new(),
                os_name: String::new(),
                kernel: String::new(),
                model: String::new(),
                cpu_model: String::new(),
                gpu_models: Vec::new(),
                ram_total: 0,
                ram_used: 0,
                screen_width: 0,
                screen_height: 0,
                shell: String::new(),
                pkgs: 0,
                pkgman_name: String::new(),
                uptime: 0,
                image_name: None,
            };
            
            for line in reader.lines() {
                if let Ok(line) = line {
                    if let Some((key, value)) = line.split_once('=') {
                        match key {
                            "user" => info.user = value.to_string(),
                            "host" => info.host = value.to_string(),
                            "version_name" => info.os_name = value.to_string(),
                            "host_model" => info.model = value.to_string(),
                            "kernel" => info.kernel = value.to_string(),
                            "cpu" => info.cpu_model = value.to_string(),
                            "gpu" => info.gpu_models.push(value.to_string()),
                            "screen_width" => info.screen_width = value.parse().unwrap_or(0),
                            "screen_height" => info.screen_height = value.parse().unwrap_or(0),
                            "shell" => info.shell = value.to_string(),
                            "pkgs" => info.pkgs = value.parse().unwrap_or(0),
                            "pkgman_name" => info.pkgman_name = value.to_string(),
                            _ => {}
                        }
                    }
                }
            }
            
            let mut sys = sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new()
                    .with_memory(sysinfo::MemoryRefreshKind::everything())
            );
            sys.refresh_memory();
            info.ram_total = sys.total_memory() / 1024 / 1024;
            info.ram_used = sys.used_memory() / 1024 / 1024;
            info.uptime = sysinfo::System::uptime();
            
            return Some(info);
        }
    }
    
    None
}