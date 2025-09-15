use std::fs;
use std::process::Command;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

#[derive(Debug, Clone)]
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
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_all();

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

        info.get_user_host();
        info.get_os_info();
        info.get_kernel();
        info.get_model();
        info.get_cpu(&sys);
        info.get_gpu();
        info.get_memory(&sys);
        info.get_resolution();
        info.get_shell();
        info.get_packages();
        info.get_uptime(&sys);

        info
    }

    fn get_user_host(&mut self) {
        if let Ok(user) = std::env::var("USER") {
            self.user = user;
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

        #[cfg(target_os = "windows")]
        return "windows".to_string();

        #[cfg(target_os = "freebsd")]
        return "freebsd".to_string();

        #[cfg(target_os = "openbsd")]
        return "openbsd".to_string();

        "unknown".to_string()
    }

    fn get_kernel(&mut self) {
        #[cfg(unix)]
        {
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

    fn get_gpu(&mut self) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("lspci").output() {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines() {
                    if line.contains("VGA") || line.contains("3D") || line.contains("Display") {
                        if let Some(gpu) = line.split(':').nth(2) {
                            let gpu = gpu.trim().replace('[', "").replace(']', "");
                            self.gpu_models.push(gpu);
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("system_profiler")
                .args(&["SPDisplaysDataType"])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines() {
                    if line.contains("Chipset Model:") {
                        if let Some(gpu) = line.split(':').nth(1) {
                            self.gpu_models.push(gpu.trim().to_string());
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("wmic")
                .args(["path", "win32_VideoController", "get", "name"])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines().skip(1) {
                    let line = line.trim();
                    if !line.is_empty() && line != "Name" {
                        self.gpu_models.push(line.to_string());
                    }
                }
            }
        }
    }

    fn get_memory(&mut self, sys: &System) {
        self.ram_total = sys.total_memory() / 1024 / 1024;
        self.ram_used = sys.used_memory() / 1024 / 1024;
    }

    fn get_resolution(&mut self) {
        #[cfg(target_os = "linux")]
        {
            if let Ok(output) = Command::new("xrandr").output() {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines() {
                    if line.contains("current") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for (i, part) in parts.iter().enumerate() {
                            if *part == "current" && i + 2 < parts.len() {
                                if let Ok(width) = parts[i + 1].parse::<u32>() {
                                    self.screen_width = width;
                                }
                                if let Ok(height) =
                                    parts[i + 3].trim_end_matches(',').parse::<u32>()
                                {
                                    self.screen_height = height;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Ok(output) = Command::new("system_profiler")
                .args(&["SPDisplaysDataType"])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines() {
                    if line.contains("Resolution:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        for part in parts {
                            if part.contains('x') {
                                let dims: Vec<&str> = part.split('x').collect();
                                if dims.len() == 2 {
                                    if let Ok(width) = dims[0].parse::<u32>() {
                                        self.screen_width = width;
                                    }
                                    if let Ok(height) = dims[1].parse::<u32>() {
                                        self.screen_height = height;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_shell(&mut self) {
        if let Ok(shell) = std::env::var("SHELL") {
            if let Some(shell_name) = shell.split('/').next_back() {
                self.shell = shell_name.to_string();
            }
        }
    }

    fn get_packages(&mut self) {
        let package_managers = [
            ("apt", "apt list --installed 2>/dev/null | wc -l"),
            ("pacman", "pacman -Qq 2>/dev/null | wc -l"),
            ("dnf", "dnf list installed 2>/dev/null | wc -l"),
            ("brew", "brew list 2>/dev/null | wc -l"),
            ("snap", "snap list 2>/dev/null | wc -l"),
            ("flatpak", "flatpak list 2>/dev/null | wc -l"),
            ("scoop", "powershell -Command \"(scoop list | Select-Object -Skip 1 | Measure-Object).Count\""),
        ];

        let mut total_pkgs = 0;
        let mut pkg_managers = Vec::new();

        for (name, cmd) in &package_managers {
            if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
                if let Ok(count) = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .parse::<u32>()
                {
                    if count > 0 {
                        total_pkgs += count;
                        pkg_managers.push(format!("{} ({})", count, name));
                    }
                }
            }
        }

        self.pkgs = total_pkgs;
        self.pkgman_name = pkg_managers.join(", ");
    }

    fn get_uptime(&mut self, _sys: &System) {
        self.uptime = System::uptime();
    }
}
