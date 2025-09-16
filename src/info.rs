#![allow(unreachable_code)]

use crate::config::Configuration;
use std::fs::{self, read_dir};
use std::path::Path;
use std::process::Command;
use std::thread;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

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
        let mut sys = System::new_with_specifics(
            RefreshKind::default()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_cpu_all();
        sys.refresh_memory();

        self.get_user_host_fast();
        if self.os_name.is_empty() {
            self.get_os_info();
        }
        self.get_kernel_fast();
        self.get_resolution();
        self.get_model();
        self.get_cpu(&sys);
        self.get_memory(&sys);
        self.get_shell();
        self.get_uptime(&sys);

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
            self.kernel = format!("Windows NT {}", std::env::consts::OS);
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
    }

    fn get_cpu(&mut self, sys: &System) {
        if let Some(cpu) = sys.cpus().first() {
            self.cpu_model = cpu.brand().to_string();

            if self.cpu_model.is_empty() {
                self.cpu_model = format!("{} Cores", sys.cpus().len());
            }
        }
    }

    fn get_memory(&mut self, sys: &System) {
        self.ram_total = sys.total_memory() / 1024 / 1024;
        self.ram_used = sys.used_memory() / 1024 / 1024;
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

    fn get_uptime(&mut self, _sys: &System) {
        self.uptime = System::uptime();
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
        let mut gpus = Vec::<String>::new();
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

    Vec::<String>::new()
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
        if let Ok(output) = Command::new("wmic")
            .args([
                "desktopmonitor",
                "get",
                "screenheight,screenwidth",
                "/format:csv",
            ])
            .output()
        {
            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                if line.contains(',') && !line.starts_with("Node,") {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 3 {
                        if let (Ok(h), Ok(w)) = (
                            parts[1].trim().parse::<u32>(),
                            parts[2].trim().parse::<u32>(),
                        ) {
                            if w > 0 && h > 0 {
                                return (w, h);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = Command::new("powershell")
            .args(["-Command", "Get-WmiObject -Class Win32_VideoController | Select-Object CurrentHorizontalResolution, CurrentVerticalResolution | Format-List"])
            .output()
        {
            let output = String::from_utf8_lossy(&output.stdout);
            let mut width = None;
            let mut height = None;

            for line in output.lines() {
                let line = line.trim();
                if let Some(rest) = line.strip_prefix("CurrentHorizontalResolution : ") {
                    width = rest.parse::<u32>().ok();
                } else if let Some(rest) = line.strip_prefix("CurrentVerticalResolution : ") {
                    height = rest.parse::<u32>().ok();
                }

                if let (Some(w), Some(h)) = (width, height) {
                    if w > 0 && h > 0 {
                        return (w, h);
                    }
                }
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
