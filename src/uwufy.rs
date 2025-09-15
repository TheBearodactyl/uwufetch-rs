pub fn uwu_name(os_name: &mut String) {
    let uwufied = match os_name.as_str() {
        "alpine" => "Nyalpine",
        "amogos" => "AmogOwOS",
        "android" => "Nyandroid",
        "arch" => "Nyarch Linuwu",
        "arcolinux" => "ArcOwO Linuwu",
        "artix" => "Nyartix Linuwu",
        "debian" => "Debinyan",
        "devuan" => "Devunyan",
        "deepin" => "Dewepyn",
        "endeavouros" | "EndeavourOS" => "endeavOwO",
        "fedora" => "Fedowa",
        "femboyos" => "FemboyOWOS",
        "gentoo" => "GentOwO",
        "gnu" => "gnUwU",
        "guix" => "gnUwU gUwUix",
        "linuxmint" => "LinUWU Miwint",
        "manjaro" => "Myanjawo",
        "manjaro-arm" => "Myanjawo AWM",
        "neon" => "KDE NeOwOn",
        "nixos" => "nixOwOs",
        "opensuse-leap" => "OwOpenSUSE Leap",
        "opensuse-tumbleweed" => "OwOpenSUSE Tumbleweed",
        "pop" => "PopOwOS",
        "raspbian" => "RaspNyan",
        "rocky" => "Wocky Linuwu",
        "slackware" => "Swackwawe",
        "solus" => "sOwOlus",
        "ubuntu" => "Uwuntu",
        "void" => "OwOid",
        "xerolinux" => "xuwulinux",
        "freebsd" => "FweeBSD",
        "openbsd" => "OwOpenBSD",
        "macos" => "macOwOS",
        "ios" => "iOwOS",
        "windows" => "WinyandOwOws",
        _ => "unknown",
    };
    
    *os_name = uwufied.to_string();
}

pub fn uwu_kernel(kernel: &mut String) {
    let replacements = [
        ("Linux", "Linuwu"),
        ("linux", "linuwu"),
        ("alpine", "Nyalpine"),
        ("amogos", "AmogOwOS"),
        ("android", "Nyandroid"),
        ("arch", "Nyarch Linuwu"),
        ("artix", "Nyartix Linuwu"),
        ("debian", "Debinyan"),
        ("deepin", "Dewepyn"),
        ("endeavouros", "endeavOwO"),
        ("EndeavourOS", "endeavOwO"),
        ("fedora", "Fedowa"),
        ("femboyos", "FemboyOWOS"),
        ("gentoo", "GentOwO"),
        ("gnu", "gnUwU"),
        ("guix", "gnUwU gUwUix"),
        ("linuxmint", "LinUWU Miwint"),
        ("manjaro", "Myanjawo"),
        ("manjaro-arm", "Myanjawo AWM"),
        ("neon", "KDE NeOwOn"),
        ("nixos", "nixOwOs"),
        ("opensuse-leap", "OwOpenSUSE Leap"),
        ("opensuse-tumbleweed", "OwOpenSUSE Tumbleweed"),
        ("pop", "PopOwOS"),
        ("raspbian", "RaspNyan"),
        ("rocky", "Wocky Linuwu"),
        ("slackware", "Swackwawe"),
        ("solus", "sOwOlus"),
        ("ubuntu", "Uwuntu"),
        ("void", "OwOid"),
        ("xerolinux", "xuwulinux"),
        ("freebsd", "FweeBSD"),
        ("openbsd", "OwOpenBSD"),
        ("macos", "macOwOS"),
        ("ios", "iOwOS"),
        ("windows", "WinyandOwOws"),
    ];
    
    for (from, to) in &replacements {
        if kernel.contains(from) {
            *kernel = kernel.replace(from, to);
        }
    }
}

pub fn uwu_hw(hw: &mut String) {
    let replacements = [
        ("lenovo", "LenOwO"),
        ("Lenovo", "LenOwO"),
        ("cpu", "CPUwU"),
        ("CPU", "CPUwU"),
        ("core", "Cowe"),
        ("Core", "Cowe"),
        ("gpu", "GPUwU"),
        ("GPU", "GPUwU"),
        ("graphics", "Gwaphics"),
        ("Graphics", "Gwaphics"),
        ("corporation", "COwOpowation"),
        ("Corporation", "COwOpowation"),
        ("nvidia", "NyaVIDIA"),
        ("NVIDIA", "NyaVIDIA"),
        ("mobile", "Mwobile"),
        ("Mobile", "Mwobile"),
        ("intel", "Inteww"),
        ("Intel", "Inteww"),
        ("celeron", "Celewon"),
        ("Celeron", "Celewon"),
        ("radeon", "Radenyan"),
        ("Radeon", "Radenyan"),
        ("geforce", "GeFOwOce"),
        ("GeForce", "GeFOwOce"),
        ("raspberry", "Nyasberry"),
        ("Raspberry", "Nyasberry"),
        ("broadcom", "Bwoadcom"),
        ("Broadcom", "Bwoadcom"),
        ("motorola", "MotOwOwa"),
        ("Motorola", "MotOwOwa"),
        ("proliant", "ProLinyant"),
        ("ProLiant", "ProLinyant"),
        ("poweredge", "POwOwEdge"),
        ("PowerEdge", "POwOwEdge"),
        ("apple", "Nyapple"),
        ("Apple", "Nyapple"),
        ("electronic", "ElectrOwOnic"),
        ("Electronic", "ElectrOwOnic"),
        ("processor", "Pwocessow"),
        ("Processor", "Pwocessow"),
        ("microsoft", "MicOwOsoft"),
        ("Microsoft", "MicOwOsoft"),
        ("ryzen", "Wyzen"),
        ("Ryzen", "Wyzen"),
        ("advanced", "Adwanced"),
        ("Advanced", "Adwanced"),
        ("micro", "Micwo"),
        ("Micro", "Micwo"),
        ("devices", "Dewices"),
        ("Devices", "Dewices"),
        ("inc.", "Nyanc."),
        ("Inc.", "Nyanc."),
        ("lucienne", "Lucienyan"),
        ("Lucienne", "Lucienyan"),
        ("tuxedo", "TUWUXEDO"),
        ("TUXEDO", "TUWUXEDO"),
        ("aura", "Uwura"),
        ("Aura", "Uwura"),
    ];
    
    for (from, to) in &replacements {
        *hw = hw.replace(from, to);
    }
}

pub fn uwu_pkgman(pkgman: &mut String) {
    let replacements = [
        ("brew-cask", "bwew-cawsk"),
        ("brew-cellar", "bwew-cewwaw"),
        ("emerge", "emewge"),
        ("flatpak", "fwatpakkies"),
        ("pacman", "pacnyan"),
        ("port", "powt"),
        ("snap", "snyap"),
    ];
    
    for (from, to) in &replacements {
        *pkgman = pkgman.replace(from, to);
    }
}

pub fn uwufy_all(info: &mut crate::info::SystemInfo) {
    uwu_kernel(&mut info.kernel);
    uwu_hw(&mut info.cpu_model);
    uwu_hw(&mut info.model);
    for gpu in &mut info.gpu_models {
        uwu_hw(gpu);
    }
    uwu_pkgman(&mut info.pkgman_name);
}