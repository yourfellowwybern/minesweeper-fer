use macroquad::prelude::*;

// ---------- Palet warna (tema gelap modern) ----------
const BG: Color = Color::new(0.07, 0.08, 0.11, 1.0);
const PANEL: Color = Color::new(0.11, 0.13, 0.18, 1.0);
const CELL_HIDDEN: Color = Color::new(0.20, 0.24, 0.33, 1.0);
const CELL_HOVER: Color = Color::new(0.28, 0.33, 0.45, 1.0);
const CELL_OPEN: Color = Color::new(0.13, 0.15, 0.21, 1.0);
const ACCENT: Color = Color::new(0.38, 0.55, 1.0, 1.0);
const TEXT: Color = Color::new(0.90, 0.92, 0.97, 1.0);
const TEXT_DIM: Color = Color::new(0.55, 0.60, 0.72, 1.0);
const RED: Color = Color::new(0.95, 0.32, 0.38, 1.0);
const GREEN: Color = Color::new(0.30, 0.85, 0.55, 1.0);

const NUMBER_COLORS: [Color; 9] = [
    TEXT,
    Color::new(0.40, 0.65, 1.00, 1.0), // 1
    Color::new(0.35, 0.85, 0.50, 1.0), // 2
    Color::new(1.00, 0.45, 0.45, 1.0), // 3
    Color::new(0.65, 0.50, 1.00, 1.0), // 4
    Color::new(1.00, 0.65, 0.30, 1.0), // 5
    Color::new(0.30, 0.85, 0.85, 1.0), // 6
    Color::new(0.90, 0.90, 0.90, 1.0), // 7
    Color::new(0.60, 0.60, 0.65, 1.0), // 8
];

const HEADER_H: f32 = 100.0;

#[derive(Clone, Copy, PartialEq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    fn params(self) -> (usize, usize, usize) {
        // (kolom, baris, ranjau)
        match self {
            Difficulty::Easy => (9, 9, 10),
            Difficulty::Medium => (16, 16, 40),
            Difficulty::Hard => (30, 16, 99),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Difficulty::Easy => "Mudah",
            Difficulty::Medium => "Sedang",
            Difficulty::Hard => "Sulit",
        }
    }
}

#[derive(Clone, Copy, Default)]
struct Cell {
    mine: bool,
    revealed: bool,
    flagged: bool,
    adj: u8,
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Ready,
    Playing,
    Won,
    Lost,
}

struct Game {
    diff: Difficulty,
    cols: usize,
    rows: usize,
    mines: usize,
    cells: Vec<Cell>,
    state: State,
    start: f64,
    elapsed: f64,
    revealed: usize,
    hit: Option<(usize, usize)>,
}

impl Game {
    fn new(diff: Difficulty) -> Self {
        let (cols, rows, mines) = diff.params();
        Game {
            diff,
            cols,
            rows,
            mines,
            cells: vec![Cell::default(); cols * rows],
            state: State::Ready,
            start: 0.0,
            elapsed: 0.0,
            revealed: 0,
            hit: None,
        }
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }

    fn neighbors(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        let mut out = Vec::with_capacity(8);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < self.cols && (ny as usize) < self.rows {
                    out.push((nx as usize, ny as usize));
                }
            }
        }
        out
    }

    fn flags_count(&self) -> usize {
        self.cells.iter().filter(|c| c.flagged).count()
    }

    /// Tempatkan ranjau setelah klik pertama supaya klik pertama selalu aman.
    fn place_mines(&mut self, sx: usize, sy: usize) {
        let mut safe = vec![(sx, sy)];
        safe.extend(self.neighbors(sx, sy));
        let mut placed = 0;
        while placed < self.mines {
            let x = rand::gen_range(0, self.cols as i32) as usize;
            let y = rand::gen_range(0, self.rows as i32) as usize;
            let i = self.idx(x, y);
            if self.cells[i].mine || safe.contains(&(x, y)) {
                continue;
            }
            self.cells[i].mine = true;
            placed += 1;
        }
        for y in 0..self.rows {
            for x in 0..self.cols {
                let n = self
                    .neighbors(x, y)
                    .iter()
                    .filter(|&&(a, b)| self.cells[self.idx(a, b)].mine)
                    .count();
                let i = self.idx(x, y);
                self.cells[i].adj = n as u8;
            }
        }
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if matches!(self.state, State::Won | State::Lost) {
            return;
        }
        if self.state == State::Ready {
            self.place_mines(x, y);
            self.state = State::Playing;
            self.start = get_time();
        }
        let i = self.idx(x, y);
        if self.cells[i].flagged || self.cells[i].revealed {
            return;
        }
        if self.cells[i].mine {
            self.cells[i].revealed = true;
            self.hit = Some((x, y));
            self.state = State::Lost;
            self.elapsed = get_time() - self.start;
            return;
        }
        // flood fill
        let mut stack = vec![(x, y)];
        while let Some((cx, cy)) = stack.pop() {
            let ci = self.idx(cx, cy);
            if self.cells[ci].revealed || self.cells[ci].flagged || self.cells[ci].mine {
                continue;
            }
            self.cells[ci].revealed = true;
            self.revealed += 1;
            if self.cells[ci].adj == 0 {
                stack.extend(self.neighbors(cx, cy));
            }
        }
        if self.revealed == self.cols * self.rows - self.mines {
            self.state = State::Won;
            self.elapsed = get_time() - self.start;
            for c in self.cells.iter_mut() {
                if c.mine {
                    c.flagged = true;
                }
            }
        }
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if !matches!(self.state, State::Playing | State::Ready) {
            return;
        }
        let i = self.idx(x, y);
        if !self.cells[i].revealed {
            self.cells[i].flagged = !self.cells[i].flagged;
        }
    }

    /// Klik pada angka yang flag di sekitarnya sudah cukup -> buka semua tetangga lainnya.
    fn chord(&mut self, x: usize, y: usize) {
        let c = self.cells[self.idx(x, y)];
        if self.state != State::Playing || !c.revealed || c.adj == 0 {
            return;
        }
        let ns = self.neighbors(x, y);
        let f = ns.iter().filter(|&&(a, b)| self.cells[self.idx(a, b)].flagged).count();
        if f == c.adj as usize {
            for (a, b) in ns {
                self.reveal(a, b);
            }
        }
    }

    fn time(&self) -> f64 {
        match self.state {
            State::Ready => 0.0,
            State::Playing => get_time() - self.start,
            _ => self.elapsed,
        }
    }
}

// ---------- Helper gambar ----------
fn text_center(t: &str, cx: f32, cy: f32, size: u16, color: Color) {
    let d = measure_text(t, None, size, 1.0);
    draw_text(t, cx - d.width / 2.0, cy + d.height / 2.0, size as f32, color);
}

fn button(r: Rect, label: &str, active: bool, mouse: Vec2) -> bool {
    let hover = r.contains(mouse);
    let col = if active {
        ACCENT
    } else if hover {
        CELL_HOVER
    } else {
        CELL_HIDDEN
    };
    draw_rectangle(r.x, r.y, r.w, r.h, col);
    text_center(label, r.x + r.w / 2.0, r.y + r.h / 2.0, 22, TEXT);
    hover && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_flag(cx: f32, cy: f32, s: f32) {
    draw_rectangle(cx - s * 0.05, cy - s * 0.30, s * 0.10, s * 0.62, TEXT);
    draw_triangle(
        vec2(cx + s * 0.05, cy - s * 0.30),
        vec2(cx + s * 0.05, cy + s * 0.05),
        vec2(cx - s * 0.30, cy - s * 0.12),
        RED,
    );
    draw_rectangle(cx - s * 0.22, cy + s * 0.30, s * 0.44, s * 0.08, TEXT);
}

fn draw_mine(cx: f32, cy: f32, s: f32, color: Color) {
    let r = s * 0.20;
    for k in 0..4 {
        let a = k as f32 * std::f32::consts::FRAC_PI_4;
        let (dx, dy) = (a.cos() * s * 0.32, a.sin() * s * 0.32);
        draw_line(cx - dx, cy - dy, cx + dx, cy + dy, 2.5, color);
    }
    draw_circle(cx, cy, r, color);
    draw_circle(cx - r * 0.35, cy - r * 0.35, r * 0.28, WHITE);
}

const BEST_FILE: &str = "best_times.txt";

fn diff_index(d: Difficulty) -> usize {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Medium => 1,
        Difficulty::Hard => 2,
    }
}

/// Format file: tiga baris (mudah, sedang, sulit); baris kosong = belum ada rekor.
fn load_best() -> [Option<u32>; 3] {
    let mut best = [None; 3];
    if let Ok(s) = std::fs::read_to_string(BEST_FILE) {
        for (i, line) in s.lines().take(3).enumerate() {
            best[i] = line.trim().parse().ok();
        }
    }
    best
}

fn save_best(best: &[Option<u32>; 3]) {
    let s: Vec<String> = best
        .iter()
        .map(|b| b.map(|v| v.to_string()).unwrap_or_default())
        .collect();
    let _ = std::fs::write(BEST_FILE, s.join("\n"));
}

fn start_game(diff: Difficulty) -> Game {
    Game::new(diff)
}

fn window_icon() -> macroquad::miniquad::conf::Icon {
    macroquad::miniquad::conf::Icon {
        small: include_bytes!(concat!(env!("OUT_DIR"), "/icon16.rgba")).to_owned(),
        medium: include_bytes!(concat!(env!("OUT_DIR"), "/icon32.rgba")).to_owned(),
        big: include_bytes!(concat!(env!("OUT_DIR"), "/icon64.rgba")).to_owned(),
    }
}

fn window_conf() -> Conf {
    Conf {
        icon: Some(window_icon()),
        window_title: "Minesweeper Modern".to_owned(),
        window_width: 1000,
        window_height: 720,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    rand::srand((get_time() * 1_000_000.0) as u64 ^ 0x9E37_79B9);
    let mut game = start_game(Difficulty::Medium);
    let mut overlay = 0.0f32; // animasi overlay menang/kalah
    let mut best = load_best();
    let mut recorded = false;
    let mut new_record = false;

    loop {
        let sw = screen_width();
        let sh = screen_height();
        let m = vec2(mouse_position().0, mouse_position().1);
        clear_background(BG);

        // ----- Header -----
        draw_rectangle(0.0, 0.0, sw, HEADER_H, PANEL);
        let mut bx = 20.0;
        let mut new_diff = None;
        for d in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            if button(Rect::new(bx, 15.0, 90.0, 36.0), d.label(), game.diff == d, m) {
                new_diff = Some(d);
            }
            bx += 100.0;
        }
        let restart = button(Rect::new(20.0, 58.0, 150.0, 30.0), "Ulang (R)", false, m);

        let left = game.mines as i32 - game.flags_count() as i32;
        text_center(&format!("Ranjau: {}", left), sw / 2.0 - 130.0, 40.0, 32, TEXT);
        text_center(&format!("Waktu: {}", game.time() as u32), sw / 2.0 + 130.0, 40.0, 32, TEXT);
        let best_txt = match best[diff_index(game.diff)] {
            Some(t) => format!("Terbaik: {} dtk", t),
            None => "Terbaik: -".to_string(),
        };
        text_center(&best_txt, sw - 110.0, 40.0, 24, TEXT_DIM);
        text_center(
            "Klik kiri: buka | Klik kanan: bendera | Klik angka: chord | 1/2/3: level",
            sw / 2.0,
            78.0,
            18,
            TEXT_DIM,
        );

        // ----- Papan -----
        let avail_w = sw - 40.0;
        let avail_h = sh - HEADER_H - 40.0;
        let cs = (avail_w / game.cols as f32)
            .min(avail_h / game.rows as f32)
            .floor()
            .min(52.0);
        let bw = cs * game.cols as f32;
        let bh = cs * game.rows as f32;
        let ox = ((sw - bw) / 2.0).floor();
        let oy = (HEADER_H + (sh - HEADER_H - bh) / 2.0).floor();

        let hover_cell = if m.x >= ox && m.y >= oy && m.x < ox + bw && m.y < oy + bh {
            Some((((m.x - ox) / cs) as usize, ((m.y - oy) / cs) as usize))
        } else {
            None
        };

        // ----- Input -----
        if is_key_pressed(KeyCode::Key1) {
            new_diff = Some(Difficulty::Easy);
        }
        if is_key_pressed(KeyCode::Key2) {
            new_diff = Some(Difficulty::Medium);
        }
        if is_key_pressed(KeyCode::Key3) {
            new_diff = Some(Difficulty::Hard);
        }
        if let Some(d) = new_diff {
            game = start_game(d);
            overlay = 0.0;
            recorded = false;
            new_record = false;
        } else if restart || is_key_pressed(KeyCode::R) {
            game = start_game(game.diff);
            overlay = 0.0;
            recorded = false;
            new_record = false;
        } else if let Some((cx, cy)) = hover_cell {
            if is_mouse_button_pressed(MouseButton::Left) {
                let c = game.cells[game.idx(cx, cy)];
                if c.revealed {
                    game.chord(cx, cy);
                } else {
                    game.reveal(cx, cy);
                }
            }
            if is_mouse_button_pressed(MouseButton::Right) {
                game.toggle_flag(cx, cy);
            }
            if is_mouse_button_pressed(MouseButton::Middle) {
                game.chord(cx, cy);
            }
        }

        // ----- Catat rekor saat menang -----
        if game.state == State::Won && !recorded {
            recorded = true;
            let t = game.elapsed as u32;
            let i = diff_index(game.diff);
            if best[i].map_or(true, |b| t < b) {
                best[i] = Some(t);
                new_record = true;
                save_best(&best);
            }
        }

        // ----- Gambar sel -----
        let gap = 2.0;
        let over = matches!(game.state, State::Lost | State::Won);
        for y in 0..game.rows {
            for x in 0..game.cols {
                let c = game.cells[game.idx(x, y)];
                let px = ox + x as f32 * cs;
                let py = oy + y as f32 * cs;
                let (cx, cy) = (px + cs / 2.0, py + cs / 2.0);
                let hovered = hover_cell == Some((x, y)) && !over;

                let show_mine = game.state == State::Lost && c.mine && !c.flagged;
                if c.revealed || show_mine {
                    let bg = if game.hit == Some((x, y)) { RED } else { CELL_OPEN };
                    draw_rectangle(px + gap / 2.0, py + gap / 2.0, cs - gap, cs - gap, bg);
                    if c.mine {
                        let col = if game.hit == Some((x, y)) { WHITE } else { RED };
                        draw_mine(cx, cy, cs, col);
                    } else if c.adj > 0 {
                        text_center(
                            &c.adj.to_string(),
                            cx,
                            cy,
                            (cs * 0.65) as u16,
                            NUMBER_COLORS[c.adj as usize],
                        );
                    }
                } else {
                    let col = if hovered { CELL_HOVER } else { CELL_HIDDEN };
                    draw_rectangle(px + gap / 2.0, py + gap / 2.0, cs - gap, cs - gap, col);
                    // highlight tipis di atas untuk kesan 3D
                    draw_rectangle(px + gap / 2.0, py + gap / 2.0, cs - gap, 3.0, Color::new(1.0, 1.0, 1.0, 0.08));
                    if c.flagged {
                        draw_flag(cx, cy, cs);
                        if game.state == State::Lost && !c.mine {
                            draw_line(px + cs * 0.2, py + cs * 0.2, px + cs * 0.8, py + cs * 0.8, 3.0, RED);
                            draw_line(px + cs * 0.8, py + cs * 0.2, px + cs * 0.2, py + cs * 0.8, 3.0, RED);
                        }
                    }
                }
            }
        }

        // ----- Overlay menang / kalah -----
        if over {
            overlay = (overlay + get_frame_time() * 2.5).min(1.0);
            let (msg, col) = if game.state == State::Won {
                (if new_record { "REKOR BARU!" } else { "KAMU MENANG!" }, GREEN)
            } else {
                ("GAME OVER", RED)
            };
            let bar_h = 90.0;
            let by = sh / 2.0 - bar_h / 2.0;
            draw_rectangle(0.0, by, sw, bar_h, Color::new(0.0, 0.0, 0.0, 0.75 * overlay));
            text_center(msg, sw / 2.0, by + 36.0, 52, Color::new(col.r, col.g, col.b, overlay));
            text_center(
                &format!("Waktu {} detik - tekan R untuk main lagi", game.elapsed as u32),
                sw / 2.0,
                by + 70.0,
                22,
                Color::new(TEXT.r, TEXT.g, TEXT.b, overlay),
            );
        }

        next_frame().await;
    }
}
