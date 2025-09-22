#![allow(unreachable_code)]

use crate::config::Configuration;
use std::env;
use std::fs::{self, read_dir};
use std::path::Path;
use std::process::Command;
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub user: String,
    pub host: String,
    pub os_name: String,
    pub kernel: String,
    pub model: String,
    pub cpu_model: String,
    pub gpu_models: Vec<String>,
    pub ram_total: u64,
    pub ram_used: u64,
    pub screen_width: u32,
    pub screen_height: u32,
    pub shell: String,
    pub pkgs: u32,
    pub pkgman_name: String,
    pub uptime: u64,
    pub image_name: Option<String>,
}

impl SystemInfo {
    pub fn populate(&mut self, config: &Configuration) {
        self.get_user_host_fast();
        if self.os_name.is_empty() {
            self.get_os_info();
        }
        self.get_kernel_fast();
        self.get_resolution();
        self.get_model();
        self.get_cpu();
        self.get_memory();
        self.get_shell();
        self.get_uptime();

        let gpu_handle = if config.show_gpu {
            Some(thread::spawn(detect_gpus))
        } else {
            None
        };
        let res_handle = if config.show_resolution {
            Some(thread::spawn(detect_resolution))
        } else {
            None
        };
        let pkgs_handle = if config.show_pkgs {
            Some(thread::spawn(detect_packages_fast))
        } else {
            None
        };

        if let Some(h) = gpu_handle {
            if let Ok(gpus) = h.join() {
                self.gpu_models = gpus;
            }
        }
        if let Some(h) = res_handle {
            if let Ok((w, hgt)) = h.join() {
                self.screen_width = w;
                self.screen_height = hgt;
            }
        }
        if let Some(h) = pkgs_handle {
            if let Ok((total, label)) = h.join() {
                self.pkgs = total;
                self.pkgman_name = label;
            }
        }
    }

    fn get_user_host_fast(&mut self) {
        if let Ok(user) = std::env::var("USER") {
            self.user = user;
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(hostname) = fs::read_to_string("/proc/sys/kernel/hostname") {
                self.host = hostname.trim().to_string();
                return;
            }
        }
        if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
            self.host = hostname.trim().to_string();
        } else if let Ok(output) = Command::new("hostname").output() {
            self.host = String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    fn get_os_info(&mut self) {
        self.os_name = Self::detect_distro();
    }

    fn detect_distro() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("ID=") {
                        return line[3..].trim_matches('"').to_string();
                    }
                }
            }

            if Path::new("/etc/debian_version").exists() {
                return "debian".to_string();
            }
            if Path::new("/etc/arch-release").exists() {
                return "arch".to_string();
            }
            if Path::new("/etc/fedora-release").exists() {
                return "fedora".to_string();
            }
        }

        #[cfg(target_os = "macos")]
        return "macos".to_string();

        #[cfg(target_os = "freebsd")]
        return "freebsd".to_string();

        #[cfg(target_os = "openbsd")]
        return "openbsd".to_string();

        #[cfg(target_os = "windows")]
        return "windows".to_string();

        "unknown".to_string()
    }

    fn get_kernel_fast(&mut self) {
        #[cfg(unix)]
        {
            #[cfg(target_os = "linux")]
            {
                if let Ok(r) = fs::read_to_string("/proc/sys/kernel/osrelease") {
                    self.kernel = r.trim().to_string();
                    return;
                }
            }
            if let Ok(output) = Command::new("uname").arg("-r").output() {
                self.kernel = String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        #[cfg(windows)]
        {
            unsafe {
                use windows::core::HSTRING;
                use windows::Win32::System::Registry::*;

                let key_path = HSTRING::from("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
                let mut hkey = Default::default();

                if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &key_path, Some(0), KEY_READ, &mut hkey)
                    .is_ok()
                {
                    let build_name = HSTRING::from("CurrentBuildNumber");
                    let mut build_buffer = [0u8; 256];
                    let mut build_size = build_buffer.len() as u32;

                    if RegQueryValueExW(
                        hkey,
                        &build_name,
                        None,
                        None,
                        Some(build_buffer.as_mut_ptr()),
                        Some(&mut build_size),
                    )
                    .is_ok()
                    {
                        if let Ok(build_str) =
                            std::str::from_utf8(&build_buffer[..build_size as usize - 1])
                        {
                            self.kernel =
                                format!("Windows NT Build {}", build_str.trim_end_matches('\0'));
                            return;
                        }
                    }
                }
            }

            self.kernel = "Windows NT".to_string();
        }
    }

    fn get_model(&mut self) {
        #[cfg(target_os = "linux")]
        {
            let model_files = [
                "/sys/devices/virtual/dmi/id/product_version",
                "/sys/devices/virtual/dmi/id/product_name",
                "/sys/devices/virtual/dmi/id/board_name",
            ];

            for file in &model_files {
                if let Ok(content) = fs::read_to_string(file) {
                    let content = content.trim();
                    if !content.is_empty() && content != "To Be Filled By O.E.M." {
                        self.model = content.to_string();
                        return;
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl").arg("hw.model").output() {
                let model = String::from_utf8_lossy(&output.stdout);
                if let Some(model) = model.split(':').nth(1) {
                    self.model = model.trim().to_string();
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            unsafe {
                use windows::core::HSTRING;
                use windows::Win32::System::Registry::*;

                let key_path =
                    HSTRING::from("SYSTEM\\CurrentControlSet\\Control\\SystemInformation");
                let mut hkey = Default::default();

                if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &key_path, Some(0), KEY_READ, &mut hkey)
                    .is_ok()
                {
                    let model_name = HSTRING::from("SystemProductName");
                    let mut model_buffer = [0u8; 512];
                    let mut model_size = model_buffer.len() as u32;

                    if RegQueryValueExW(
                        hkey,
                        &model_name,
                        None,
                        None,
                        Some(model_buffer.as_mut_ptr()),
                        Some(&mut model_size),
                    )
                    .is_ok()
                    {
                        if let Ok(model_str) =
                            std::str::from_utf8(&model_buffer[..model_size as usize - 1])
                        {
                            let model = model_str.trim_end_matches('\0').trim();
                            if !model.is_empty() {
                                self.model = model.to_string();
                                return;
                            }
                        }
                    }
                }
            }

            if let Ok(output) = Command::new("wmic")
                .args(["computersystem", "get", "model"])
                .output()
            {
                let model = String::from_utf8_lossy(&output.stdout);
                for line in model.lines().skip(1) {
                    let line = line.trim();
                    if !line.is_empty() {
                        self.model = line.to_string();
                        break;
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let model_files = [
                "/sys/devices/virtual/dmi/id/product_version",
                "/sys/devices/virtual/dmi/id/product_name",
                "/sys/devices/virtual/dmi/id/board_name",
            ];

            for file in &model_files {
                if let Ok(content) = fs::read_to_string(file) {
                    let content = content.trim();
                    if !content.is_empty() && content != "To Be Filled By O.E.M." {
                        self.model = content.to_string();
                        return;
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl").arg("hw.model").output() {
                let model = String::from_utf8_lossy(&output.stdout);
                if let Some(model) = model.split(':').nth(1) {
                    self.model = model.trim().to_string();
                }
            }
        }
    }

    fn get_cpu(&mut self) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                use windows::core::HSTRING;
                use windows::Win32::System::Registry::*;

                let key_path = HSTRING::from("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
                let mut hkey = Default::default();

                if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &key_path, Some(0), KEY_READ, &mut hkey)
                    .is_ok()
                {
                    let processor_name = HSTRING::from("ProcessorNameString");
                    let mut cpu_buffer = [0u8; 512];
                    let mut cpu_size = cpu_buffer.len() as u32;

                    if RegQueryValueExW(
                        hkey,
                        &processor_name,
                        None,
                        None,
                        Some(cpu_buffer.as_mut_ptr()),
                        Some(&mut cpu_size),
                    )
                    .is_ok()
                    {
                        if let Ok(cpu_str) =
                            std::str::from_utf8(&cpu_buffer[..cpu_size as usize - 1])
                        {
                            self.cpu_model = cpu_str.trim_end_matches('\0').trim().to_string();
                            return;
                        }
                    }
                }
            }

            if let Ok(output) = Command::new("wmic").args(["cpu", "get", "name"]).output() {
                let cpu = String::from_utf8_lossy(&output.stdout);
                for line in cpu.lines().skip(1) {
                    let line = line.trim();
                    if !line.is_empty() && line != "Name" {
                        self.cpu_model = line.to_string();
                        return;
                    }
                }
            }

            self.cpu_model = "Unknown CPU".to_string();
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
                let mut brand = String::new();
                let mut count = 0u32;

                for line in content.lines() {
                    if line.starts_with("model name") {
                        if let Some(name) = line.split(':').nth(1) {
                            brand = name.trim().to_string();
                        }
                        count += 1;
                    }
                }

                if !brand.is_empty() {
                    self.cpu_model = brand;
                } else {
                    self.cpu_model = format!("{} Cores", count);
                }
                return;
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("sysctl")
                .arg("machdep.cpu.brand_string")
                .output()
            {
                let brand = String::from_utf8_lossy(&output.stdout);
                if let Some(brand) = brand.split(':').nth(1) {
                    self.cpu_model = brand.trim().to_string();
                    return;
                }
            }
        }

        self.cpu_model = "Unknown CPU".to_string();
    }

    fn get_memory(&mut self) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                use windows::Win32::System::SystemInformation::{
                    GlobalMemoryStatusEx, MEMORYSTATUSEX,
                };

                let mut memstatus = MEMORYSTATUSEX {
                    dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                    ..Default::default()
                };

                if GlobalMemoryStatusEx(&mut memstatus).is_ok() {
                    self.ram_total = (memstatus.ullTotalPhys / 1024 / 1024) as u64;
                    self.ram_used =
                        ((memstatus.ullTotalPhys - memstatus.ullAvailPhys) / 1024 / 1024) as u64;
                    return;
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
                                    self.ram_total = total / 1024;
                                    self.ram_used = (total - free) / 1024;
                                    return;
                                }
                            }
                        }
                    }
                }
            }
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

    fn get_resolution(&mut self) {
        self.screen_width = detect_resolution().0;
        self.screen_height = detect_resolution().1;
    }

    fn get_shell(&mut self) {
        if let Ok(shell) = std::env::var("SHELL") {
            if let Some(shell_name) = shell.rsplit('/').next() {
                self.shell = shell_name.to_string();
            }
        }
    }

    fn get_uptime(&mut self) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                use windows::Win32::System::SystemInformation::GetTickCount64;

                let tick_count = GetTickCount64();
                self.uptime = tick_count / 1000;
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
}

fn detect_gpus() -> Vec<String> {
    #[cfg(target_os = "linux")]
    {
        if which("lspci") {
            if let Ok(out) = Command::new("lspci").args(["-mm", "-nn"]).output() {
                let mut gpus = Vec::<String>::new();
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    if line.contains("VGA compatible controller")
                        || line.contains("3D controller")
                        || line.contains("Display controller")
                    {
                        let parts: Vec<&str> = line.split('"').collect();
                        if parts.len() >= 10 {
                            let vendor = parts.get(5).unwrap_or(&"").trim();
                            let device = parts.get(7).unwrap_or(&"").trim();
                            let combo = format!("{} {}", vendor, device).trim().to_string();
                            if !combo.is_empty() {
                                gpus.push(combo);
                                continue;
                            }
                        }
                        gpus.push(line.to_string());
                    }
                }
                return gpus;
            }
        }

        let mut gpus = Vec::<String>::new();
        if let Ok(entries) = read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if !name.starts_with("card") {
                    continue;
                }
                let uevent = entry.path().join("device/uevent");
                if let Ok(txt) = fs::read_to_string(uevent) {
                    let mut driver = None;
                    for line in txt.lines() {
                        if let Some(val) = line.strip_prefix("DRIVER=") {
                            driver = Some(val.trim().to_string());
                        }
                    }
                    if let Some(d) = driver {
                        if !gpus.contains(&d) {
                            gpus.push(d);
                        }
                    }
                }
            }
        }
        return gpus;
    }

    #[cfg(target_os = "macos")]
    {
        let mut gpus = Vec::<String>::new();
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
        {
            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                if let Some(rest) = line.strip_prefix("      Chipset Model: ") {
                    gpus.push(rest.trim().to_string());
                }
            }
        }
        return gpus;
    }

    #[cfg(target_os = "windows")]
    {
        let mut gpus: Vec<String> = Vec::new();

        unsafe {
            use windows::core::HSTRING;
            use windows::Win32::System::Registry::*;

            let key_path = HSTRING::from(
                "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}",
            );
            let mut hkey = HKEY::default();

            if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &key_path, Some(0), KEY_READ, &mut hkey).is_ok() {
                for i in 0..10 {
                    let subkey_name = HSTRING::from(format!("{:04}", i));
                    let mut subkey = HKEY::default();

                    if RegOpenKeyExW(hkey, &subkey_name, Some(0), KEY_READ, &mut subkey).is_ok() {
                        let desc_name = HSTRING::from("DriverDesc");
                        let mut desc_buffer = [0u8; 512];
                        let mut desc_size = desc_buffer.len() as u32;

                        if RegQueryValueExW(
                            subkey,
                            &desc_name,
                            None,
                            None,
                            Some(desc_buffer.as_mut_ptr()),
                            Some(&mut desc_size),
                        )
                        .is_ok()
                        {
                            if let Ok(desc_str) =
                                std::str::from_utf8(&desc_buffer[..desc_size as usize - 1])
                            {
                                let desc = desc_str.trim_end_matches('\0').trim();
                                if !desc.is_empty() && !gpus.contains(&desc.to_string()) {
                                    gpus.push(desc.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        if !gpus.is_empty() {
            return gpus;
        }

        if let Ok(output) = Command::new("wmic")
            .args(["path", "win32_VideoController", "get", "name"])
            .output()
        {
            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines().skip(1) {
                let line = line.trim();
                if !line.is_empty() && line != "Name" {
                    gpus.push(line.to_string());
                }
            }
        }

        return gpus;
    }

    Vec::new()
}

fn detect_resolution() -> (u32, u32) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(v) = fs::read_to_string("/sys/class/graphics/fb0/virtual_size") {
            let mut it = v.trim().split(',');
            if let (Some(w), Some(h)) = (it.next(), it.next()) {
                if let (Ok(ww), Ok(hh)) = (w.parse::<u32>(), h.parse::<u32>()) {
                    return (ww, hh);
                }
            }
        }
        if std::env::var("DISPLAY").is_ok() && which("xrandr") {
            if let Ok(out) = Command::new("xrandr").arg("--current").output() {
                let s = String::from_utf8_lossy(&out.stdout);
                for line in s.lines() {
                    if let Some(idx) = line.find("current") {
                        let tail = &line[idx + "current".len()..];
                        let mut it = tail.split_whitespace();
                        let w = it.next().and_then(|t| t.parse::<u32>().ok());
                        let _x = it.next();
                        let h = it
                            .next()
                            .map(|t| t.trim_end_matches(','))
                            .and_then(|t| t.parse::<u32>().ok());
                        if let (Some(ww), Some(hh)) = (w, h) {
                            return (ww, hh);
                        }
                    }
                }
            }
        }
        return (0u32, 0u32);
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
        {
            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                if let Some(rest) = line.strip_prefix("          Resolution: ") {
                    let mut parts = rest.split_whitespace();
                    if let (Some(w), Some(_x), Some(h)) = (parts.next(), parts.next(), parts.next())
                    {
                        if let (Ok(ww), Ok(hh)) = (w.parse::<u32>(), h.parse::<u32>()) {
                            return (ww, hh);
                        }
                    }
                }
            }
        }
        return (0u32, 0u32);
    }

    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::UI::WindowsAndMessaging::{
                GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
            };

            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);

            if width > 0 && height > 0 {
                return (width as u32, height as u32);
            }
        }

        return (0u32, 0u32);
    }

    (0u32, 0u32)
}

fn detect_packages_fast() -> (u32, String) {
    let mut total: u32 = 0;
    let mut labels: Vec<String> = Vec::new();

    #[cfg(target_os = "linux")]
    {
        if Path::new("/var/lib/dpkg/status").exists() {
            if let Ok(s) = fs::read_to_string("/var/lib/dpkg/status") {
                let count = s
                    .lines()
                    .filter(|l| *l == "Status: install ok installed")
                    .count() as u32;
                if count > 0 {
                    total += count;
                    labels.push(format!("{} (dpkg)", count));
                }
            }
        }

        if Path::new("/var/lib/pacman/local").exists() {
            let mut count = 0u32;
            if let Ok(rd) = read_dir("/var/lib/pacman/local") {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_dir() && p.join("desc").exists() {
                        count += 1;
                    }
                }
            }
            if count > 0 {
                total += count;
                labels.push(format!("{} (pacman)", count));
            }
        }

        if which("rpm") {
            if let Ok(out) = Command::new("rpm")
                .args(["-qa", "--qf", "%{NAME}\n"])
                .output()
            {
                let count = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count() as u32;
                if count > 0 {
                    total += count;
                    labels.push(format!("{} (rpm)", count));
                }
            }
        }

        if which("flatpak") {
            if let Ok(out) = Command::new("flatpak")
                .args(["list", "--app", "--columns=application"])
                .output()
            {
                let count = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count() as u32;
                if count > 0 {
                    total += count;
                    labels.push(format!("{} (flatpak)", count));
                }
            }
        }

        if which("snap") {
            if let Ok(out) = Command::new("snap").args(["list"]).output() {
                let count = String::from_utf8_lossy(&out.stdout)
                    .lines()
                    .skip(1)
                    .filter(|l| !l.trim().is_empty())
                    .count() as u32;
                if count > 0 {
                    total += count;
                    labels.push(format!("{} (snap)", count));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut count = 0u32;
        for base in ["/opt/homebrew/Cellar", "/usr/local/Cellar"] {
            if Path::new(base).exists() {
                if let Ok(rd) = read_dir(base) {
                    for e in rd.flatten() {
                        if e.path().is_dir() {
                            count += 1;
                        }
                    }
                }
            }
        }
        if count > 0 {
            total += count;
            labels.push(format!("{} (brew)", count));
        }

        let mut casks = 0u32;
        for base in ["/opt/homebrew/Caskroom", "/usr/local/Caskroom"] {
            if Path::new(base).exists() {
                if let Ok(rd) = read_dir(base) {
                    for e in rd.flatten() {
                        if e.path().is_dir() {
                            casks += 1;
                        }
                    }
                }
            }
        }
        if casks > 0 {
            total += casks;
            labels.push(format!("{} (casks)", casks));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(home) = std::env::var("USERPROFILE") {
            let p = format!(r"{}\scoop\apps", home);
            if Path::new(&p).exists() {
                if let Ok(rd) = read_dir(p) {
                    let count = rd.flatten().filter(|e| e.path().is_dir()).count() as u32;
                    if count > 0 {
                        total += count;
                        labels.push(format!("{} (scoop)", count));
                    }
                }
            }
        }
    }

    if total == 0 {
        return (0u32, String::new());
    }
    (total, labels.join(", "))
}

#[cfg(not(target_os = "windows"))]
fn which(cmd: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        for path in env::split_paths(&paths) {
            let full_path = path.join(cmd);
            if full_path.is_file() {
                return true;
            }
            #[cfg(windows)]
            {
                let exe_path = full_path.with_extension("exe");
                if exe_path.is_file() {
                    return true;
                }
            }
        }
    }
    false
}
