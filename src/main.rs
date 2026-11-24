use std::env;
use std::io::{self, Write};

use signal_hook::consts::{SIGINT, SIGWINCH};
use signal_hook::iterator::Signals;
use terminal_size::{Height, Width, terminal_size_of};

const HIDE_CURSOR: &str = "\x1b[?25l";
const SHOW_CURSOR: &str = "\x1b[?25h";
const CLEAR_SCREEN: &str = "\x1b[2J\x1b[H";
const CURSOR_HOME: &str = "\x1b[H";
const CLEAR_TO_EOL: &str = "\x1b[K";
const RESET: &str = "\x1b[0m";
const BG_TERMINAL_DEFAULT: &str = "\x1b[49m";

const COLORS: [&str; 7] = [
    "\x1b[41m", "\x1b[42m", "\x1b[43m", "\x1b[44m", "\x1b[45m", "\x1b[46m", "\x1b[47m",
];

const PATCH_W: usize = 6;
const PATCH_H: usize = 7;
const GAP: usize = 2;
const STEP_SIZE: usize = 2;
const TOTAL_W: usize = COLORS.len() * PATCH_W + (COLORS.len() - 1) * GAP;

const FALLBACK_SIZE: (usize, usize) = (80, 24);

fn get_terminal_size() -> (usize, usize) {
    let from_env = |name| {
        env::var(name)
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .filter(|&n| n > 0)
    };
    let (cols, rows) = match terminal_size_of(io::stdout()) {
        Some((Width(w), Height(h))) => (w.into(), h.into()),
        None => FALLBACK_SIZE,
    };
    (
        from_env("COLUMNS").unwrap_or(cols),
        from_env("LINES").unwrap_or(rows),
    )
}

fn push_spaces(out: &mut String, count: usize) {
    out.extend(std::iter::repeat_n(' ', count));
}

fn push_patch_row(out: &mut String, start_x: usize, row: usize) {
    out.push_str(BG_TERMINAL_DEFAULT);
    push_spaces(out, start_x);

    for (i, color) in COLORS.iter().enumerate() {
        if i > 0 {
            out.push_str(BG_TERMINAL_DEFAULT);
            push_spaces(out, GAP);
        }
        if row == 0 {
            out.push_str(color);
            push_spaces(out, PATCH_W - STEP_SIZE);
            out.push_str(BG_TERMINAL_DEFAULT);
            push_spaces(out, STEP_SIZE);
        } else if row == PATCH_H - 1 {
            out.push_str(BG_TERMINAL_DEFAULT);
            push_spaces(out, STEP_SIZE);
            out.push_str(color);
            push_spaces(out, PATCH_W - STEP_SIZE);
        } else {
            out.push_str(color);
            push_spaces(out, PATCH_W);
        }
    }
}

fn render_palette(cols: usize, rows: usize) -> String {
    let start_x = cols.saturating_sub(TOTAL_W) / 2;
    let start_y = rows.saturating_sub(PATCH_H) / 2;

    let mut out = String::from(CURSOR_HOME);
    out.push_str(HIDE_CURSOR);
    for y in 0..rows.saturating_sub(1) {
        if (start_y..start_y + PATCH_H).contains(&y) {
            push_patch_row(&mut out, start_x, y - start_y);
        }
        out.push_str(RESET);
        out.push_str(CLEAR_TO_EOL);
        out.push('\n');
    }
    out.push_str(RESET);
    out.push_str(CLEAR_TO_EOL);
    out
}

fn write_stdout(text: &str) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(text.as_bytes())?;
    stdout.flush()
}

fn draw_palette() -> io::Result<()> {
    let (cols, rows) = get_terminal_size();
    write_stdout(&render_palette(cols, rows))
}

fn main() -> io::Result<()> {
    let mut signals = Signals::new([SIGINT, SIGWINCH])?;

    write_stdout(CLEAR_SCREEN)?;
    draw_palette()?;

    for signal in signals.forever() {
        match signal {
            SIGWINCH => draw_palette()?,
            _ => break,
        }
    }

    write_stdout(&format!("{RESET}{SHOW_CURSOR}{CLEAR_SCREEN}"))
}
