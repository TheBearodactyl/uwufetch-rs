use crate::info::SystemInfo;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

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

            for line in reader.lines().map_while(Result::ok) {
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

            info.ram_total = get_mem().0;
            info.ram_used = get_mem().1;
            info.uptime = get_uptime();

            return Some(info);
        }
    }

    None
}

fn get_uptime() -> u64 {
    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::System::SystemInformation::GetTickCount64;

            let tick_count = GetTickCount64();
            tick_count / 1000
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/uptime") {
            if let Some(uptime_str) = content.split_whitespace().next() {
                if let Ok(uptime_f) = uptime_str.parse::<f64>() {
                    self.uptime = uptime_f as u64;
                    return;
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("sysctl").arg("kern.boottime").output() {
            let boottime = String::from_utf8_lossy(&output.stdout);
            if let Ok(output) = Command::new("uptime").output() {
                let uptime_str = String::from_utf8_lossy(&output.stdout);
                if uptime_str.contains("days") {
                    self.uptime = 86400;
                }
            }
        }
    }
}

fn get_mem() -> (u64, u64) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        unsafe {
            use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

            let mut memstatus = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };

            if GlobalMemoryStatusEx(&mut memstatus).is_ok() {
                let ram_total = (memstatus.ullTotalPhys / 1024 / 1024) as u64;
                let ram_used =
                    ((memstatus.ullTotalPhys - memstatus.ullAvailPhys) / 1024 / 1024) as u64;

                return (ram_total, ram_used);
            }
        }

        if let Ok(output) = Command::new("wmic")
            .args([
                "OS",
                "get",
                "TotalVisibleMemorySize,FreePhysicalMemory",
                "/format:csv",
            ])
            .output()
        {
            let mem = String::from_utf8_lossy(&output.stdout);
            for line in mem.lines().skip(1) {
                if !line.trim().is_empty() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        if let Ok(free) = parts[1].parse::<u64>() {
                            if let Ok(total) = parts[2].parse::<u64>() {
                                let ram_total = total / 1024;
                                let ram_used = (total - free) / 1024;

                                return (ram_total, ram_used);
                            }
                        }
                    }
                }
            }
        }

        (0, 0)
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = fs::read_to_string("/proc/meminfo") {
            let mut total = 0u64;
            let mut available = 0u64;

            for line in content.lines() {
                if line.starts_with("MemTotal:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        total = val.parse().unwrap_or(0);
                    }
                } else if line.starts_with("MemAvailable:") {
                    if let Some(val) = line.split_whitespace().nth(1) {
                        available = val.parse().unwrap_or(0);
                    }
                }
            }

            self.ram_total = total / 1024;
            self.ram_used = (total - available) / 1024;
            return;
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        if let Ok(output) = Command::new("sysctl").arg("hw.memsize").output() {
            let mem = String::from_utf8_lossy(&output.stdout);
            if let Some(size) = mem.split(':').nth(1) {
                if let Ok(bytes) = size.trim().parse::<u64>() {
                    self.ram_total = bytes / 1024 / 1024;
                }
            }
        }

        if let Ok(output) = Command::new("vm_stat").output() {
            let vm_output = String::from_utf8_lossy(&output.stdout);
            let mut active = 0u64;
            let mut wired = 0u64;
            let mut compressed = 0u64;

            for line in vm_output.lines() {
                if let Some(val) = line.split_whitespace().last() {
                    let val = val.trim_end_matches('.');
                    if let Ok(pages) = val.parse::<u64>() {
                        if line.contains("Pages active:") {
                            active = pages;
                        } else if line.contains("Pages wired down:") {
                            wired = pages;
                        } else if line.contains("Pages occupied by compressor:") {
                            compressed = pages;
                        }
                    }
                }
            }

            let page_size = 4096u64;
            self.ram_used = (active + wired + compressed) * page_size / 1024 / 1024;
        }
        return;
    }
}
