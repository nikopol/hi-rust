use std::env;

use hi::colors::*;
use hi::fonts::*;
use hi::sys::*;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let cfg = Config::parse(&args).map_err(|e| return e)?;

    if let Some(font) = get_font(cfg.font_name) {
        return font.print(
            &cfg.text,
            &cfg.info_lines,
            cfg.small,
            cfg.maxcols,
            &cfg.fg_color,
            &cfg.bg_color,
            cfg.colormode,
        );
    } else {
        return Err("unknown font".to_string());
    }
}

#[derive(Default)]
struct Config {
    fg: Option<String>,
    bg: Option<String>,
    info: Option<String>,
    small: bool,
    text: String,
    font_name: Option<String>,
    info_lines: Vec<String>,
    colormode: u16,
    ssh: Option<String>,
    maxcols: Option<usize>,
    cheapmode: bool,
    rootmode: bool,
    fg_color: ColorVec,
    bg_color: Option<ColorVec>,
}

impl Config {
    fn help() -> String {
        let fonts = font_names();
        let colors = color_names();
        let name = env!("CARGO_PKG_NAME");
        let version = env!("CARGO_PKG_VERSION");
        format!(
            "{name} {version}

syntax: {name} [-options] [text]

default text is your hostname

options:
-fg=color         foreground color/gradient
-bg=color         background color
-c[olor]=1|16|256 set color mode
-1|16|256         set color mode as well
-m[ono]           set color mode to 1
-font=default     font name (available: {fonts})
-s[mall]          set small mode (default font only)
-i[nfo]=kumic     choose info to display
-ssh=remote       ssh hostname

infos can be given as follow:
k=kernel   u=uptime   c=cpumodel
m=memory   i=ip

color can be given as follow:
name          eg: green
d,d,d         eg: 0,255,0
h:h:h         eg: 0:ff:0
hhhhhh        eg: 00ff00
hhh           eg: 0f0

gradient can be given as follow:
color-color   eg: white-blue
                  fff-00f

available colors: {colors}
available fonts: {fonts}
"
        )
    }

    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut cfg = Config::default();
        let mut i = 0;
        let mut text_parts: Vec<String> = Vec::new();
        while i < args.len() {
            let arg = &args[i];
            if let Some(opt) = arg.strip_prefix('-') {
                let (name, value_in_arg) = Self::split_option(opt);
                match name.to_lowercase().as_str() {
                    "fg" | "foreground" => {
                        let value = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                        cfg.fg = value;
                    }
                    "bg" | "background" => {
                        let value = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                        cfg.bg = value;
                    }
                    "font" | "fontname" => {
                        cfg.font_name = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                    }
                    "i" | "info" => {
                        let value = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                        cfg.info = value;
                    }
                    "1" | "16" | "256" => cfg.colormode = name.parse().unwrap(),
                    "m" | "mono" => cfg.colormode = 1,
                    "c" | "color" | "colors" => {
                        let value = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                        cfg.colormode = match value.unwrap_or("?".to_string()).as_str() {
                            "mono" => 1,
                            "1" => 1,
                            "16" => 16,
                            "256" => 256,
                            _ => return Err(String::from("colors can only be mono, 1, 16 or 256")),
                        };
                    }
                    "s" | "small" => cfg.small = true,
                    "h" | "help" => return Err(Self::help()),
                    "ssh" => {
                        cfg.ssh = value_in_arg
                            .map(str::to_string)
                            .or_else(|| Self::next_arg(args, &mut i));
                    }
                    other if other.is_empty() => {}
                    other => {
                        return Err(format!("unknown option -{other}"));
                    }
                }
            } else {
                text_parts.push(arg.clone());
            }
            i += 1;
        }

        cfg.cheapmode = env::var("TERM").map(|t| t == "linux").unwrap_or(false);
        cfg.rootmode = env::var("USER").map(|u| u == "root").unwrap_or(false);
        if cfg.cheapmode {
            cfg.small = false;
            cfg.colormode = 16;
        } else if cfg.colormode == 0 {
            cfg.colormode = 256;
        }

        let fg_spec = cfg.fg.as_ref().map(|fg| fg.as_str()).unwrap_or_else(|| {
            if cfg.cheapmode {
                if cfg.rootmode {
                    "red"
                } else {
                    "cyan"
                }
            } else if cfg.rootmode {
                "f88-fff"
            } else {
                "66c-fff"
            }
        });

        cfg.fg_color = rgb(fg_spec, cfg.colormode)?;
        cfg.bg_color = match cfg.bg.as_ref() {
            Some(bg) => Some(rgb(&bg, cfg.colormode)?),
            None => None,
        };
        let info_fg_color = rgb("grey", cfg.colormode)?;

        cfg.text = if text_parts.is_empty() {
            let hostname = shell("hostname", &cfg.ssh).unwrap_or_else(|| "localhost".into());
            let clean = hostname.trim();
            clean.split('.').next().unwrap_or(clean).to_string()
        } else {
            text_parts.join(" ")
        };

        let info_spec = if let Some(spec) = cfg.info.as_ref() {
            spec
        } else if cfg.small {
            &String::from("imu")
        } else {
            &String::from("kicmu")
        };

        let mut infos = Vec::new();
        for ch in info_spec.to_lowercase().chars() {
            if let Some(info) = match ch {
                'k' => kernel_info(cfg.cheapmode, &cfg.ssh, &info_fg_color, cfg.colormode),
                'u' => uptime_info(cfg.cheapmode, &cfg.ssh, &info_fg_color, cfg.colormode),
                'i' => ip_info(cfg.cheapmode, &cfg.ssh, &info_fg_color, cfg.colormode),
                'c' => cpu_info(cfg.cheapmode, &cfg.ssh, &info_fg_color, cfg.colormode),
                'm' => mem_info(cfg.cheapmode, &cfg.ssh, &info_fg_color, cfg.colormode),
                _ => None,
            } {
                infos.push(info);
            } else {
                infos.push(format!("info [{ch}] not available"))
            }
        }
        cfg.info_lines = infos;

        Ok(cfg)
    }

    fn split_option(arg: &str) -> (&str, Option<&str>) {
        if let Some(idx) = arg.find('=') {
            (&arg[..idx], Some(&arg[idx + 1..]))
        } else {
            (arg, None)
        }
    }

    fn next_arg(args: &[String], idx: &mut usize) -> Option<String> {
        let next = *idx + 1;
        if next < args.len() {
            *idx += 1;
            Some(args[next].clone())
        } else {
            None
        }
    }
}
