use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use crate::info::SystemInfo;

#[derive(Debug, Clone)]
pub struct Configuration {
    pub show_user: bool,
    pub show_os: bool,
    pub show_host: bool,
    pub show_kernel: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_resolution: bool,
    pub show_shell: bool,
    pub show_pkgs: bool,
    pub show_uptime: bool,
    pub show_colors: bool,
    pub show_image: bool,
    pub gpu_indexes: Vec<usize>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            show_user: true,
            show_os: true,
            show_host: true,
            show_kernel: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_resolution: true,
            show_shell: true,
            show_pkgs: true,
            show_uptime: true,
            show_colors: true,
            show_image: false,
            gpu_indexes: vec![],
        }
    }
}

impl Configuration {
    pub fn parse_config(user_info: &mut SystemInfo) -> Self {
        let mut config = Configuration::default();
        
        let config_path = Self::find_config_file();
        if let Some(path) = config_path {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let line = line.trim();
                        if line.is_empty() || line.starts_with('#') {
                            continue;
                        }
                        
                        if let Some((key, value)) = line.split_once('=') {
                            let key = key.trim();
                            let value = value.trim().trim_matches('"');
                            
                            match key {
                                "distro" => user_info.os_name = value.to_string(),
                                "image" => {
                                    let mut image_path = value.to_string();
                                    if image_path.starts_with('~') {
                                        if let Ok(home) = std::env::var("HOME") {
                                            image_path = image_path.replacen('~', &home, 1);
                                        }
                                    }
                                    user_info.image_name = Some(image_path);
                                    config.show_image = true;
                                }
                                "user" => config.show_user = value == "true",
                                "os" => config.show_os = value != "false",
                                "host" => config.show_host = value != "false",
                                "kernel" => config.show_kernel = value != "false",
                                "cpu" => config.show_cpu = value != "false",
                                "gpu" => {
                                    if let Ok(idx) = value.parse::<usize>() {
                                        config.gpu_indexes.push(idx);
                                    }
                                }
                                "gpus" => config.show_gpu = value != "false",
                                "ram" => config.show_ram = value != "false",
                                "resolution" => config.show_resolution = value != "false",
                                "shell" => config.show_shell = value != "false",
                                "pkgs" => config.show_pkgs = value != "false",
                                "uptime" => config.show_uptime = value != "false",
                                "colors" => config.show_colors = value != "false",
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        config
    }
    
    fn find_config_file() -> Option<PathBuf> {
        if let Ok(home) = std::env::var("HOME") {
            let user_config = PathBuf::from(home).join(".config/uwufetch/config");
            if user_config.exists() {
                return Some(user_config);
            }
        }
        
        if let Ok(prefix) = std::env::var("PREFIX") {
            let prefixed_config = PathBuf::from(prefix).join("etc/uwufetch/config");
            if prefixed_config.exists() {
                return Some(prefixed_config);
            }
        }
        
        let system_config = PathBuf::from("/etc/uwufetch/config");
        if system_config.exists() {
            return Some(system_config);
        }
        
        None
    }
}