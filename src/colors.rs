const COLOR_TABLE: [(&str, (u8, u8, u8)); 17] = [
    ("black", (0x00, 0x00, 0x00)),
    ("bright_black", (0x82, 0x82, 0x82)),
    ("red", (0xbf, 0x79, 0x79)),
    ("bright_red", (0xf4, 0xa4, 0x5f)),
    ("green", (0x97, 0xb2, 0x6b)),
    ("bright_green", (0xc5, 0xf7, 0x79)),
    ("yellow", (0xcd, 0xcd, 0xa1)),
    ("bright_yellow", (0xff, 0xff, 0xaf)),
    ("blue", (0x86, 0xa2, 0xbe)),
    ("bright_blue", (0x98, 0xaf, 0xd9)),
    ("magenta", (0x96, 0x3c, 0x59)),
    ("bright_magenta", (0xef, 0x9e, 0xbe)),
    ("cyan", (0x7f, 0x9f, 0x7f)),
    ("bright_cyan", (0x71, 0xbe, 0xbe)),
    ("white", (0xde, 0xde, 0xde)),
    ("bright_white", (0xff, 0xff, 0xff)),
    ("grey", (0x82, 0x82, 0x82)),
];

const SYSCOLOR_TABLE: [(&str, u8); 16] = [
    ("black", 30),
    ("bright_black", 90),
    ("red", 31),
    ("bright_red", 91),
    ("green", 32),
    ("bright_green", 92),
    ("yellow", 33),
    ("bright_yellow", 93),
    ("blue", 34),
    ("bright_blue", 94),
    ("magenta", 35),
    ("bright_magenta", 95),
    ("cyan", 36),
    ("bright_cyan", 96),
    ("white", 37),
    ("bright_white", 97),
];

pub fn color_names() -> String {
    let names: Vec<&'static str> = COLOR_TABLE.iter().map(|(name, _)| *name).collect();
    names.join(",")
}

fn color_by_name(spec: &str) -> Option<(u8, u8, u8)> {
    let lower = spec.to_lowercase();
    COLOR_TABLE
        .iter()
        .find(|(name, _)| *name == lower)
        .map(|(_, rgb)| *rgb)
}

fn color_by_exact_name(name: &str) -> Option<(u8, u8, u8)> {
    COLOR_TABLE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, rgb)| *rgb)
}

pub type ColorVec = Vec<i32>;

pub fn rgb(spec: &str, colormode: u16) -> Result<ColorVec, String> {
    if let Some(idx) = spec.find('-') {
        let (left, right) = spec.split_at(idx);
        let right = &right[1..];
        let start = rgb(left, colormode)?;
        let end = rgb(right, colormode)?;
        if start.len() < 3 || end.len() < 3 {
            return Ok(start);
        }
        return Ok(vec![start[0], start[1], start[2], end[0], end[1], end[2]]);
    }
    if let Some(rgb) = parse_decimal_rgb(spec) {
        return Ok(convert_rgb(rgb, colormode));
    }
    if let Some(rgb) = parse_hex_rgb(spec) {
        return Ok(convert_rgb(rgb, colormode));
    }
    if let Some(rgb) = parse_color_name(spec) {
        return Ok(convert_rgb(rgb, colormode));
    }
    Err(format!("invalid color {spec}"))
}

fn parse_decimal_rgb(spec: &str) -> Option<(u8, u8, u8)> {
    let parts: Vec<_> = spec.split(',').collect();
    if parts.len() == 3 {
        let r = parts[0].parse::<u8>().ok()?;
        let g = parts[1].parse::<u8>().ok()?;
        let b = parts[2].parse::<u8>().ok()?;
        return Some((r, g, b));
    }
    let parts: Vec<_> = spec.split(':').collect();
    if parts.len() == 3 {
        let r = u8::from_str_radix(parts[0], 16).ok()?;
        let g = u8::from_str_radix(parts[1], 16).ok()?;
        let b = u8::from_str_radix(parts[2], 16).ok()?;
        return Some((r, g, b));
    }
    None
}

fn parse_hex_rgb(spec: &str) -> Option<(u8, u8, u8)> {
    if spec.len() == 6 {
        let r = u8::from_str_radix(&spec[0..2], 16).ok()?;
        let g = u8::from_str_radix(&spec[2..4], 16).ok()?;
        let b = u8::from_str_radix(&spec[4..6], 16).ok()?;
        return Some((r, g, b));
    }
    if spec.len() == 3 {
        let r = u8::from_str_radix(&spec[0..1], 16).ok()?;
        let g = u8::from_str_radix(&spec[1..2], 16).ok()?;
        let b = u8::from_str_radix(&spec[2..3], 16).ok()?;
        return Some((r * 17, g * 17, b * 17));
    }
    None
}

fn parse_color_name(spec: &str) -> Option<(u8, u8, u8)> {
    color_by_name(spec)
}

fn convert_rgb(rgb: (u8, u8, u8), colormode: u16) -> ColorVec {
    if colormode == 256 {
        vec![rgb.0 as i32, rgb.1 as i32, rgb.2 as i32]
    } else {
        vec![nearest_ansi(rgb) as i32]
    }
}

fn nearest_ansi(rgb: (u8, u8, u8)) -> u8 {
    let mut best: u8 = 0;
    let mut delta = i32::MAX;
    for (name, code) in SYSCOLOR_TABLE.iter() {
        if let Some((r, g, b)) = color_by_exact_name(name) {
            let d = (rgb.0 as i32 - r as i32).abs()
                + (rgb.1 as i32 - g as i32).abs()
                + (rgb.2 as i32 - b as i32).abs();
            if d < delta {
                delta = d;
                best = *code;
            }
        }
    }
    best
}

pub fn gradient_color(color: &ColorVec, ratio: f32) -> ColorVec {
    if color.len() == 6 {
        let clamped = ratio.clamp(0.0, 1.0);
        let r = color[0] + ((color[3] - color[0]) as f32 * clamped) as i32;
        let g = color[1] + ((color[4] - color[1]) as f32 * clamped) as i32;
        let b = color[2] + ((color[5] - color[2]) as f32 * clamped) as i32;
        vec![r, g, b]
    } else {
        color.clone()
    }
}

pub fn color_sequence(fg: &ColorVec, bg: &Option<ColorVec>, colormode: u16) -> String {
    let mut out = String::new();
    out.push_str(&term_color(false, fg, colormode));
    if let Some(color) = bg {
        out.push_str(&term_color(true, &color, colormode));
    }
    out
}

fn term_color(is_bg: bool, color: &ColorVec, colormode: u16) -> String {
    if colormode == 16 {
        let mut code = color.get(0).copied().unwrap_or(39);
        if is_bg {
            code += 10;
        }
        return format!("\x1b[{code}m");
    }
    if color.len() < 3 {
        return String::new();
    }
    let cf = 6.0 / 256.0;
    let col = 16
        + (color[0] as f32 * cf).floor() as i32 * 36
        + (color[1] as f32 * cf).floor() as i32 * 6
        + (color[2] as f32 * cf).floor() as i32;
    let pfx: u8 = if is_bg { 48 } else { 38 };
    format!("\x1b[{pfx};5;{col}m")
}

pub const COLOR_RESET: &str = "\x1b[0m";

pub fn bar(width: usize, cf: f32, colormode: u16, cheapmode: bool) -> String {
    let cf = cf.clamp(0.0, 1.0);
    let mut used = (width as f32 * cf).floor() as usize;
    if used > width {
        used = width;
    }
    let free = width.saturating_sub(used);
    let color_name = if cf > 0.8 {
        "red"
    } else if cf > 0.5 {
        "yellow"
    } else {
        "green"
    };
    let color_vec = rgb(color_name, colormode).unwrap_or_default();
    let mut out = String::new();
    out.push_str(&color_sequence(&color_vec, &None, colormode));
    let used_char = if cheapmode { "#" } else { "⯀" };
    let free_char = if cheapmode { "-" } else { "▢" };
    out.push_str(&used_char.repeat(used));
    out.push_str(COLOR_RESET);
    out.push_str(&free_char.repeat(free));
    out
}
