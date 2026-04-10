use std::process::Command;

use crate::colors::*;

#[cfg(target_os = "linux")]
pub fn kernel_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let icon: String = if cheapmode {
        String::from("")
    } else {
        String::from("🐧 ")
    };
    let label = format!("{icon}kernel");
    let uname_r = shell("uname -r", &ssh)
        .unwrap_or_else(|| "?".into())
        .replace('\n', "");
    Some(label_with_info(
        &label,
        &uname_r,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

#[cfg(target_os = "macos")]
pub fn kernel_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let icon: String = if cheapmode {
        String::from("")
    } else {
        String::from("🍎 ")
    };
    let label = format!("{icon}kernel");
    let uname_r = shell("uname -r", &ssh)
        .unwrap_or_else(|| "?".into())
        .replace('\n', "");
    Some(label_with_info(
        &label,
        &uname_r,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

pub fn uptime_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let label = format!("{}load/up", if cheapmode { "" } else { "⏳ " });
    let uptime = shell("uptime", &ssh)?;
    let up_part = uptime
        .split(" up ")
        .nth(1)
        .and_then(|rest| rest.split(',').next())
        .unwrap_or("?")
        .trim()
        .to_string();
    let load_str = uptime
        .split("load average:")
        .nth(1)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "?".into());
    let first_load = load_str
        .split(|c: char| c == ',' || c.is_whitespace())
        .find(|s| !s.is_empty())
        .unwrap_or("0");
    let first_load_val = first_load.replace(',', ".");
    let cpubrand = cpuinfo_data(ssh);
    let nbcores = cpubrand.1.max(1) as f32;
    let load_val = first_load_val.parse::<f32>().unwrap_or(0.0);
    let cf = (load_val / nbcores).clamp(0.0, 1.0);
    let bar_str = bar(10, cf, colormode, cheapmode);
    let info = format!("{bar_str} {load_str} ({up_part})");
    Some(label_with_info(
        &label,
        &info,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

pub fn ip_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let label = format!("{}ip", if cheapmode { "" } else { "🌐 " });
    let output = shell("/sbin/ip -4 addr 2>/dev/null", &ssh)
        .or_else(|| shell("/sbin/ifconfig 2>/dev/null", &ssh))
        .unwrap_or_default();
    let ips = extract_ipv4(&output);
    let public: Vec<_> = ips
        .iter()
        .filter(|ip| !is_private_ip(ip))
        .cloned()
        .collect();
    let info = if public.is_empty() {
        ips.join(" / ")
    } else {
        public.join(" / ")
    };
    Some(label_with_info(
        &label,
        &info,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

pub fn cpu_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let label = format!("{}cpu", if cheapmode { "" } else { "💻 " });
    let (brand, cores) = cpuinfo_data(ssh);
    let info = match brand {
        Some(brand) => format!("{cores} x {brand}"),
        None => "not found".into(),
    };
    Some(label_with_info(
        &label,
        &info,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

#[cfg(target_os = "linux")]
pub fn mem_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: &ColorVec,
    colormode: u16,
) -> Option<String> {
    let label = format!("{}mem", if cheapmode { "" } else { "🧠 " });
    let info = {
        let free_out = shell("free -m", &ssh).unwrap_or_default();
        let mem_line = free_out.lines().find(|l| l.starts_with("Mem"));
        if let Some(line) = mem_line {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() >= 3 {
                let total = fields[1].parse::<f32>().unwrap_or(0.0);
                let used = fields[2].parse::<f32>().unwrap_or(0.0);
                if total > 0.0 {
                    format!(
                        "{} {}mo used / {}mo",
                        bar(10, used / total, colormode, cheapmode),
                        used as i32,
                        total as i32
                    )
                } else {
                    "?".into()
                }
            } else {
                "?".into()
            }
        } else {
            "?".into()
        }
    };
    Some(label_with_info(
        &label,
        &info,
        info_fg_color,
        colormode,
        cheapmode,
    ))
}

#[cfg(target_os = "macos")]
pub fn mem_info(
    cheapmode: bool,
    ssh: &Option<String>,
    info_fg_color: Option<ColorVec>,
    colormode: u16,
) -> Option<String> {
    let label = format!("{}mem", if cheapmode { "" } else { "🧠 " });
    let info = {
        let total = shell("sysctl -n hw.memsize", &ssh)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .map(|v| v / 1024 / 1024)
            .unwrap_or(0);
        if total == 0 {
            "?".into()
        } else {
            let vm_stat = shell("vm_stat", &ssh).unwrap_or_default();
            let wired = vm_stat
                .lines()
                .find(|l| l.contains("Pages wired"))
                .and_then(|l| {
                    l.chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(0);
            let active = vm_stat
                .lines()
                .find(|l| l.contains("Pages active"))
                .and_then(|l| {
                    l.chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse::<u64>()
                        .ok()
                })
                .unwrap_or(0);
            let used = 4096 * (wired + active) / 1024 / 1024;
            format!(
                "{} {}",
                bar(10, used as f32 / total as f32, colormode, cheapmode),
                format!("{used}mo used / {total}mo")
            )
        }
    };
    Some(label_with_info(&label, &info, info_fg_color, colormode))
}

#[cfg(target_os = "linux")]
pub fn cpuinfo_data(ssh: &Option<String>) -> (Option<String>, usize) {
    let info = shell("cat /proc/cpuinfo", &ssh).unwrap_or_default();
    let mut brand = None;
    let mut nbcores = 0;
    for line in info.lines() {
        if line.to_lowercase().starts_with("model name") {
            let parts: Vec<_> = line.split(':').collect();
            if parts.len() > 1 && brand.is_none() {
                brand = Some(parts[1].trim().to_string());
            }
            nbcores += 1;
        }
    }
    if nbcores == 0 {
        nbcores = 1;
    }
    (brand, nbcores)
}

#[cfg(target_os = "macos")]
pub fn cpuinfo_data(ssh: &Option<String>) -> (Option<String>, usize) {
    let nbcores = shell("sysctl -n hw.ncpu", &ssh)
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(1);
    let brand = shell("sysctl -n machdep.cpu.brand_string", &ssh).map(|s| s.trim().to_string());
    (brand, nbcores);
}

fn label_with_info(
    label: &str,
    info: &str,
    info_fg_color: &ColorVec,
    colormode: u16,
    cheapmode: bool,
) -> String {
    let mut out = String::new();
    out.push_str(&color_sequence(info_fg_color, &None, colormode));
    let fmtlabel = if cheapmode {
        format!("{:<8}", label)
    } else {
        format!("{:<10}", label)
    };
    out.push_str(&fmtlabel);
    out.push_str(COLOR_RESET);
    out.push_str(info);
    out
}

fn extract_ipv4(text: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for token in text.split(|c: char| c.is_whitespace() || c == '/') {
        if token.matches('.').count() == 3
            && token.chars().all(|c| c.is_ascii_digit() || c == '.')
            && token != "127.0.0.1"
            && !token.ends_with(".255")
        {
            ips.push(
                token
                    .trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                    .to_string(),
            );
        }
    }
    ips
}

fn is_private_ip(ip: &str) -> bool {
    let parts: Vec<_> = ip.split('.').filter_map(|s| s.parse::<u8>().ok()).collect();
    if parts.len() != 4 {
        return false;
    }
    match parts[0] {
        10 => true,
        192 if parts[1] == 168 => true,
        172 if (16..=31).contains(&parts[1]) => true,
        _ => false,
    }
}

pub fn shell(cmd: &str, ssh: &Option<String>) -> Option<String> {
    let mut command = if let Some(host) = ssh {
        let mut c = Command::new("ssh");
        c.arg(host).arg(cmd);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(cmd);
        c
    };
    let output = command.output().ok()?;
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}
