use crate::colors::*;
use unicode_normalization::UnicodeNormalization;


/* ███ ███ ██  ███ ███ 
   █   █ █ █ █  █  █   
   ██  █ █ █ █  █  ███ 
   █   █ █ █ █  █    █ 
   █   ███ █ █  █  ███ */


fn trunc_println(line: &str, max_cols: u16, color_mode: u32, prefix: Option<&str>) {
    if max_cols == u16::MAX {
        println!("{line}");
        return;
    }
    let max = max_cols as usize;
    let mut count = 0usize;
    let mut iter = line.chars().peekable();
    if let Some(p) = &prefix {
        count = p.len();
        print!("{p}");
    }
    while let Some(&ch) = iter.peek() {
        if ch == '\x1b' {
            iter.next();
            print!("\x1b");
            for c in iter.by_ref() {
                print!("{c}");
                if c.is_ascii_alphabetic() { break; }
            }
        } else if count < max - 2 {
            print!("{ch}");
            iter.next();
            count += 1;
        } else {
            if color_mode > 1 {
                print!("{COLOR_RESET}…");
            }
            break;
        }
    }
    if color_mode > 1 {
        print!("{COLOR_RESET}");
    }
    println!("")
}

#[derive(Clone, Copy)]
pub struct Font {
    pub charset: &'static str,
    pub rows: &'static [&'static str],
    separator: Separator,
}

#[derive(Clone, Copy)]
pub enum Separator {
    Space,
    Pipe,
}

const SMALL_DOTS: [&str; 16] = [
    " ", "▘", "▝", "▀", "▖", "▌", "▞", "▛", "▗", "▚", "▐", "▜", "▃", "▙", "▟", "█",
];

impl Font {
    fn parse_font_bmp(&self) -> Vec<Vec<&'static str>> {
        self.rows
            .iter()
            .map(|row| match self.separator {
                Separator::Space => row.split_whitespace().collect(),
                Separator::Pipe => row
                    .split('|')
                    .filter(|segment| !segment.is_empty())
                    .collect(),
            })
            .collect()
    }

    fn chars_width(chars_bmp: &[Vec<&'static str>], chars_idx: &[usize]) -> usize {
        let mut width = chars_idx.len();
        if chars_bmp.is_empty() {
            return width;
        }
        for &idx in chars_idx {
            if let Some(glyph) = chars_bmp[0].get(idx) {
                width += glyph.chars().count();
            }
        }
        width
    }

    fn get_chars_idx(charset: &str, text: &str) -> Vec<usize> {
        let sanitized = text
            .to_lowercase()
            .nfd()
            .filter(|c| !('\u{0300}'..='\u{036F}').contains(c))
            .collect::<String>();
        let default_idx = charset.chars().position(|c| c == '?').unwrap_or(0);
        sanitized
            .chars()
            .map(|ch| charset.chars().position(|c| c == ch).unwrap_or(default_idx))
            .collect()
    }

    fn draw(
        chars_idx: &[usize],
        chars_bmp: &[Vec<&'static str>],
        info_lines: &[String],
        width: usize,
        fg: &ColorVec,
        bg: &Option<ColorVec>,
        color_mode: u32,
        max_cols: u16,
        prefix: Option<&str>
    ) {
        for (y, row) in chars_bmp.iter().enumerate() {
            let mut line = String::new();
            let mut r = 0usize;
            for &idx in chars_idx {
                let glyph = row.get(idx).copied().unwrap_or(" ");
                for ch in glyph.chars().chain(Some(' ')) {
                    let ch = if ch == '.' { ' ' } else { ch };
                    if color_mode > 1 {
                        let color_vec = gradient_color(fg, (r as f32) / (width.max(1) as f32));
                        line.push_str(&color_sequence(&color_vec, bg, color_mode));
                    }
                    line.push(ch);
                    r += 1;
                }
            }
            if let Some(comment) = info_lines.get(y) {
                if color_mode > 1 {
                    line.push_str(COLOR_RESET);
                }
                line.push_str(comment);
            }
            trunc_println(&line, max_cols, color_mode, prefix);
        }
    }

    fn small_draw(
        chars_idx: &[usize],
        chars_bmp: &[Vec<&'static str>],
        info_lines: &[String],
        width: usize,
        fg: &ColorVec,
        bg: &Option<ColorVec>,
        color_mode: u32,
        max_cols: u16,
        prefix: Option<&str>
    ) {
        if chars_bmp.is_empty() {
            return;
        }
        let height = chars_bmp.len();
        for y in (0..height).step_by(2) {
            let mut line = String::new();
            let mut r = 0usize;
            for &idx in chars_idx {
                let upper = chars_bmp[y].get(idx).copied().unwrap_or(" ");
                let lower = if y + 1 < height {
                    chars_bmp[y + 1].get(idx).copied().unwrap_or(" ")
                } else {
                    ""
                };
                let mut l1: Vec<char> = upper.chars().collect();
                l1.push('.');
                let mut l2: Vec<char> = lower.chars().collect();
                if !l2.is_empty() {
                    l2.push('.');
                }
                for j in (0..l1.len()).step_by(2) {
                    let mut b = 0;
                    if l1.get(j).copied().unwrap_or('.') != '.' {
                        b += 1;
                    }
                    if l1.get(j + 1).copied().unwrap_or('.') != '.' {
                        b += 2;
                    }
                    if l2.get(j).copied().unwrap_or('.') != '.' {
                        b += 4;
                    }
                    if l2.get(j + 1).copied().unwrap_or('.') != '.' {
                        b += 8;
                    }
                    if color_mode > 1 {
                        let color_vec = gradient_color(fg, (r as f32) / (width.max(1) as f32));
                        line.push_str(&color_sequence(&color_vec, bg, color_mode));
                    }
                    line.push_str(SMALL_DOTS.get(b).unwrap_or(&" "));
                    r += 1;
                }
            }
            if let Some(comment) = info_lines.get(y / 2) {
                if color_mode > 1 {
                    line.push_str(COLOR_RESET);
                }
                line.push_str(comment);
            }
            trunc_println(&line, max_cols, color_mode, prefix);
        }
    }

    pub fn print(
        &self,
        text: &String,
        info_lines: &[String],
        smallmode: bool,
        maxcols: Option<usize>,
        fg_color: &ColorVec,
        bg_color: &Option<ColorVec>,
        color_mode: u32,
        max_cols: u16,
        prefix: Option<&str>
    ) -> Result<(), String> {
        let chars_bmp = self.parse_font_bmp();
        let chars_idx = Self::get_chars_idx(self.charset, text);
        let width = Self::chars_width(&chars_bmp, &chars_idx);

        if let Some(limit) = maxcols {
            if width > limit {
                return Err(format!("text \"{text}\" too large"));
            }
        }

        if smallmode {
            Self::small_draw(
                &chars_idx,
                &chars_bmp,
                info_lines,
                width / 2,
                fg_color,
                bg_color,
                color_mode,
                max_cols,
                prefix
            );
        } else {
            Self::draw(
                &chars_idx,
                &chars_bmp,
                info_lines,
                width,
                fg_color,
                bg_color,
                color_mode,
                max_cols,
                prefix
            );
        }

        Ok(())
    }
}

struct FontEntry {
    name: &'static str,
    font: Font,
}

const FONT_ENTRIES: [FontEntry; 3] = [
    FontEntry {
        name: "default",
        font: Font {
            charset: " abcdefghijklmnopqrstuvwxyz0123456789?!#+-*\\/()[]{}.:,\"'=_",
            rows: &[
".. ███ ███ ███ ██. ███ ███ ███ █.█ █ ..█ █.█ █.. █.█ ██. ███ ███ ███ ███ ███ ███ █.█ █.█ █.█ █.█ █.█ ███ .█. ..█ ██. ██. █.. ███ █.. ███ ███ ███ ███ █ █.█ ... ... ... ... ... .█ █. .██ ██. .██ ██. . ... .. .. █.█ █ ... ...",
".. █.█ █.█ █.. █.█ █.. █.. █.. █.█ . ..█ █.█ █.. ███ █.█ █.█ █.█ █.█ █.█ █.. .█. █.█ █.█ █.█ █.█ █.█ ..█ █.█ .██ ..█ ..█ █.█ █.. █.. ..█ █.█ █.█ ..█ █ ███ .█. ... █.█ █.. ..█ █. .█ .█. .█. .█. .█. . .█. .. .█ █.█ █ ███ ...",
".. ███ ██. █.. █.█ ██. ██. █.█ ███ █ ..█ ██. █.. ███ █.█ █.█ ███ █.█ ██. ███ .█. █.█ █.█ ███ .█. ███ .█. █.█ █.█ .█. .█. ███ ██. ███ .█. ███ ███ .██ █ █.█ ███ ███ .█. .█. .█. █. .█ .█. .█. █.. ..█ . ... .. .. ... . ... ...",
".. █.█ █.█ █.. █.█ █.. █.. █.█ █.█ █ █.█ █.█ █.. █.█ █.█ █.█ █.. ███ █.█ ..█ .█. █.█ █.█ ███ █.█ .█. █.. █.█ ..█ █.. ..█ ..█ ..█ █.█ █.. █.█ ..█ ... . ███ .█. ... █.█ ..█ █.. █. .█ .█. .█. .█. .█. . .█. .█ .█ ... . ███ ...",
".. █.█ ███ ███ ███ ███ █.. ███ █.█ █ ███ █.█ ███ █.█ █.█ ███ █.. ..█ █.█ ███ .█. ███ .█. █.█ █.█ .█. ███ .█. ..█ ███ ██. ..█ ██. ███ █.. ███ ..█ .█. █ █.█ ... ... ... ... ... .█ █. .██ ██. .██ ██. █ ... █. █. ... . ... ███",
            ],
            separator: Separator::Space,
        },
    },
    FontEntry {
        name: "dots",
        font: Font { // =>
            //charset: " abcdefghijklmnopqrstuvwxyz0123456789?!#+-*\\/()[]{}.:,\"'=_",
            charset: " abcdefghijklmnopqrstuvwxyz012",
            rows: &[
".. ⡎⠉⢱ ⡎⠑⡄ ⡎⠉ ⡏⠑⡄ ⡏⠉ ⡏⠉ ⡎⠉⠁ ⡇.⢸ ⡅ ..⢸ ⡇.⢸ ⡇. ⡗⢄⡠⢺ ⡗⢄⢸ ⡎⠉⢱ ⡏⠑⡄ ⡎⠉⢱ ⡏⠑⡄ ⡠⠊⠉ ⠉⡏⠉ ⡇.⢸ ⡇.⡇ ⡇..⡇ ⡇.⢸ ⡇.⡇ ⠉⠉⡇ ⡎⠉⢱ ⡠⢺ ⠉⠑⢄ ⠉⠉⢢ ⡇.. ",
".. ⡇.⢸ ⡇.⡸ ⡇. ⡇.⢸ ⡇. ⡇. ⡇.. ⡇.⢸ ⡇ ..⢸ ⡇.⡸ ⡇. ⡇..⢸ ⡇.⢹ ⡇.⢸ ⡇.⡸ ⡇.⢸ ⡇.⡸ ⡇.. .⡇. ⡇.⢸ ⡇.⡇ ⡇..⡇ ⡇.⢸ ⡇.⡇ .⢸. ⡇.⢸ .⢸ ..⢸ ..⢸ ",
".. ⡗⠒⢺ ⡗⠪⡀ ⡇. ⡇.⢸ ⡗⠂ ⡗⠂ ⡇⠐⢢ ⡗⠒⢺ ⡇ ..⢸ ⡗⠪⡀ ⡇. ⡇..⢸ ⡇.⢸ ⡇.⢸ ⡗⠊. ⡇.⢸ ⡗⠪⡀ ⠑⠤⡀ .⡇. ⡇.⢸ ⡇.⡇ ⡇..⡇ ⢈⠶⡁ ⠈⡎. .⡇. ⡇⠶⢸ .⢸ ⢀⠔⠁ .⠤⡊ ",
".. ⡇.⢸ ⡇.⢸ ⡇. ⡇.⢸ ⡇. ⡇. ⡇.⢸ ⡇.⢸ ⡇ ..⢸ ⡇.⢸ ⡇. ⡇..⢸ ⡇.⢸ ⡇.⢸ ⡇.. ⡇.⢸ ⡇.⢸ ..⢸ .⡇. ⡇.⢸ ⢸⢸. ⡇..⡇ ⡇.⢸ .⡇. ⢸.. ⡇.⢸ .⢸ ⡎.. ..⢸ ",
".. ⡇.⢸ ⣇⡠⠃ ⢇⣀ ⣇⡠⠃ ⣇⣀ ⡇. ⢇⣀⡸ ⡇.⢸ ⡇ ⢄⡠⠃ ⡇.⢸ ⣇⣀ ⡇..⢸ ⡇.⢸ ⢇⣀⡸ ⡇.. ⢇⣑⣼ ⡇.⢸ ⣀⡠⠊ .⡇. ⢇⣀⡸ ⠘⡜. ⢣⠊⢢⠃ ⡇.⢸ .⡇. ⣇⣀⡀ ⢇⣀⡸ .⢸ ⣇⣀⣀ ⣀⣀⠜ ",
            ],
            separator: Separator::Space,
        },
    },
    FontEntry {
        name: "hashtag",
        font: Font {
            charset: " abcdefghijklmnopqrstuvwxyz0123456789?!#+-*\\/()[]{}.:,\"'=_",
            rows: &[
".. .##. ###. .### ###. #### #### .##. #..# # ...# #..# #... #...# #..# .##. ###. .##. ###. .### ##### #..# #...# #...# #...# #...# ##### .#. ..# ##. ##. #.. ### #.. ### ### ### ### # #.# ... ... ... ... ... .# #. .## ##. .## ##. . ... .. .. #.# # ... ...",
".. #..# #..# #... #..# #... #... #... #..# . ...# #..# #... ##.## ##.# #..# #..# #..# #..# #... ..#.. #..# #...# #...# .#.#. .#.#. ...#. #.# .## ..# ..# #.# #.. #.. ..# #.# #.# ..# # ### .#. ... #.# #.. ..# #. .# .#. .#. .#. .#. . .#. .. .# #.# # ### ...",
".. #### ###. #... #..# ###. ###. #.## #### # ...# ###. #... #.#.# #.## #..# ###. #..# ###. .##. ..#.. #..# #...# #.#.# ..#.. ..#.. ..#.. #.# #.# .#. .#. ### ##. ### .#. ### ### .## # #.# ### ### .#. .#. .#. #. .# .#. .#. #.. ..# . ... .. .. ... . ... ...",
".. #..# #..# #... #..# #... #... #..# #..# # #..# #..# #... #...# #..# #..# #... #.## #..# ...# ..#.. #..# .#.#. ##.## .#.#. ..#.. .#... #.# ..# #.. ..# ..# ..# #.# #.. #.# ..# ... . ### .#. ... #.# ..# #.. #. .# .#. .#. .#. .#. . .#. .# .# ... . ### ...",
".. #..# ###. .### ###. #### #... .##. #..# # .##. #..# #### #...# #..# .##. #... .### #..# ###. ..#.. .##. ..#.. #...# #...# ..#.. ##### .#. ..# ### ##. ..# ##. ### #.. ### ..# .#. # #.# ... ... ... ... ... .# #. .## ##. .## ##. # ... #. #. ... . ... ###",
            ],
            separator: Separator::Space,
        },
    },
];

pub fn get_font(font_name: Option<String>) -> Option<&'static Font> {
    let name = font_name.unwrap_or("default".into());
    FONT_ENTRIES
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(name.as_str()))
        .map(|entry| &entry.font)
}

pub fn font_names() -> String {
    let names: Vec<&'static str> = FONT_ENTRIES.iter().map(|entry| entry.name).collect();
    names.join(",")
}
