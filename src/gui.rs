use crate::error::{YppError, YppResult};
use minifb::{Key, Window, WindowOptions};

/// Maximum grid size (cells). Caps memory used by the framebuffer.
pub const MAX_CELLS: usize = 256;
/// Pixels per cell. 60x70 cells → 600x700 window at the default scale.
pub const DEFAULT_CELL_PX: usize = 10;
pub const MIN_CELLS: usize = 1;
pub const MAX_CELL_PX: usize = 32;

#[derive(Clone, Copy, Debug)]
pub struct EntitySprite {
    pub x: i32,
    pub y: i32,
    pub width: usize,
    pub height: usize,
    pub color: u32,
}

pub struct GuiRuntime {
    window: Option<Window>,
    pub cells_w: usize,
    pub cells_h: usize,
    pub cell_px: usize,
    pub bg: u32,
    buffer: Vec<u32>,
    pub entities: Vec<EntitySprite>,
    pub closed: bool,
}

impl GuiRuntime {
    pub fn new(cells_w: usize, cells_h: usize, line: usize) -> YppResult<Self> {
        let cells_w = clamp_cells(cells_w);
        let cells_h = clamp_cells(cells_h);
        let cell_px = DEFAULT_CELL_PX;
        let px_w = cells_w.saturating_mul(cell_px).max(1);
        let px_h = cells_h.saturating_mul(cell_px).max(1);
        // Guard against oversized buffers even after clamping.
        let pixels = px_w.saturating_mul(px_h);
        if pixels > MAX_CELLS.saturating_mul(MAX_CELLS).saturating_mul(MAX_CELL_PX).saturating_mul(MAX_CELL_PX) {
            return Err(YppError::gui_size_error(line, cells_w, cells_h));
        }

        let window = Window::new(
            "Y++ yGUI",
            px_w,
            px_h,
            WindowOptions {
                resize: false,
                ..WindowOptions::default()
            },
        )
        .map_err(|e| YppError::gui_window_error(line, &e.to_string()))?;

        let mut gui = Self {
            window: Some(window),
            cells_w,
            cells_h,
            cell_px,
            bg: color_named("BLACK").unwrap_or(0x000000),
            buffer: vec![0; pixels],
            entities: Vec::new(),
            closed: false,
        };
        if let Some(w) = gui.window.as_mut() {
            w.set_target_fps(60);
        }
        Ok(gui)
    }

    pub fn set_dimensions(&mut self, width: usize, height: usize, line: usize) -> YppResult<()> {
        let width = clamp_cells(width);
        let height = clamp_cells(height);
        if width == self.cells_w && height == self.cells_h {
            return Ok(());
        }
        self.cells_w = width;
        self.cells_h = height;
        let px_w = self.cells_w.saturating_mul(self.cell_px).max(1);
        let px_h = self.cells_h.saturating_mul(self.cell_px).max(1);
        self.buffer = vec![0; px_w.saturating_mul(px_h)];
        match Window::new(
            "Y++ yGUI",
            px_w,
            px_h,
            WindowOptions {
                resize: false,
                ..WindowOptions::default()
            },
        ) {
            Ok(mut w) => {
                w.set_target_fps(60);
                self.window = Some(w);
                self.closed = false;
                Ok(())
            }
            Err(e) => Err(YppError::gui_window_error(line, &e.to_string())),
        }
    }

    pub fn set_color(&mut self, color: u32) {
        self.bg = color;
    }

    pub fn upsert_player(&mut self, width: usize, height: usize) {
        let width = width.clamp(MIN_CELLS, self.cells_w.max(MIN_CELLS));
        let height = height.clamp(MIN_CELLS, self.cells_h.max(MIN_CELLS));
        if let Some(e) = self.entities.get_mut(0) {
            e.width = width;
            e.height = height;
        } else {
            let x = (self.cells_w.saturating_sub(width) / 2) as i32;
            let y = (self.cells_h.saturating_sub(height) / 2) as i32;
            self.entities.push(EntitySprite {
                x,
                y,
                width,
                height,
                color: color_named("WHITE").unwrap_or(0xFFFFFF),
            });
        }
    }

    pub fn player_mut(&mut self) -> Option<&mut EntitySprite> {
        self.entities.get_mut(0)
    }

    pub fn key_held(&self, name: &str) -> bool {
        let Some(window) = self.window.as_ref() else {
            return false;
        };
        if let Some(key) = key_from_name(name) {
            window.is_key_down(key)
        } else {
            false
        }
    }

    /// Draw the grid and present. Returns `false` when the user closed the window.
    pub fn present(&mut self, line: usize) -> YppResult<bool> {
        if self.closed {
            return Ok(false);
        }
        let Some(window) = self.window.as_mut() else {
            self.closed = true;
            return Ok(false);
        };
        if !window.is_open() {
            self.closed = true;
            self.window = None;
            return Ok(false);
        }

        let px_w = self.cells_w.saturating_mul(self.cell_px).max(1);
        let px_h = self.cells_h.saturating_mul(self.cell_px).max(1);
        if self.buffer.len() != px_w.saturating_mul(px_h) {
            self.buffer.resize(px_w.saturating_mul(px_h), self.bg);
        }
        for px in self.buffer.iter_mut() {
            *px = self.bg;
        }

        let entities = self.entities.clone();
        for ent in &entities {
            fill_rect(
                &mut self.buffer,
                px_w,
                px_h,
                self.cell_px,
                self.cells_w,
                self.cells_h,
                *ent,
            );
        }

        window
            .update_with_buffer(&self.buffer, px_w, px_h)
            .map_err(|e| YppError::gui_window_error(line, &e.to_string()))?;

        if !window.is_open() {
            self.closed = true;
            self.window = None;
            return Ok(false);
        }
        Ok(true)
    }
}

fn clamp_cells(n: usize) -> usize {
    n.clamp(MIN_CELLS, MAX_CELLS)
}

fn fill_rect(
    buffer: &mut [u32],
    px_w: usize,
    px_h: usize,
    cell_px: usize,
    cells_w: usize,
    cells_h: usize,
    ent: EntitySprite,
) {
    let x0 = ent.x.max(0) as usize;
    let y0 = ent.y.max(0) as usize;
    let x1 = (x0 + ent.width).min(cells_w);
    let y1 = (y0 + ent.height).min(cells_h);
    for cy in y0..y1 {
        for cx in x0..x1 {
            let px = cx * cell_px;
            let py = cy * cell_px;
            for dy in 0..cell_px {
                let row = py + dy;
                if row >= px_h {
                    continue;
                }
                let start = row * px_w + px;
                let end = (start + cell_px).min((row + 1) * px_w);
                for slot in &mut buffer[start..end] {
                    *slot = ent.color;
                }
            }
        }
    }
}

pub fn color_named(name: &str) -> Option<u32> {
    match name.trim().to_ascii_uppercase().as_str() {
        "BLACK" => Some(0x000000),
        "WHITE" => Some(0xFFFFFF),
        "RED" => Some(0xFF0000),
        "GREEN" => Some(0x00FF00),
        "BLUE" => Some(0x0000FF),
        "YELLOW" => Some(0xFFFF00),
        "CYAN" | "AQUA" => Some(0x00FFFF),
        "MAGENTA" | "PURPLE" => Some(0xFF00FF),
        "ORANGE" => Some(0xFFA500),
        "GRAY" | "GREY" => Some(0x808080),
        "PINK" => Some(0xFFC0CB),
        "BROWN" => Some(0x8B4513),
        "NAVY" => Some(0x000080),
        "TEAL" => Some(0x008080),
        "LIME" => Some(0x32CD32),
        "SILVER" => Some(0xC0C0C0),
        _ => None,
    }
}

pub fn color_from_value(n: f64) -> u32 {
    if !n.is_finite() {
        return 0;
    }
    (n as i64).clamp(0, 0x00FF_FFFF) as u32
}

pub fn key_from_name(name: &str) -> Option<Key> {
    let s = name.trim();
    if s.len() == 1 {
        return match s.chars().next()?.to_ascii_uppercase() {
            'A' => Some(Key::A),
            'B' => Some(Key::B),
            'C' => Some(Key::C),
            'D' => Some(Key::D),
            'E' => Some(Key::E),
            'F' => Some(Key::F),
            'G' => Some(Key::G),
            'H' => Some(Key::H),
            'I' => Some(Key::I),
            'J' => Some(Key::J),
            'K' => Some(Key::K),
            'L' => Some(Key::L),
            'M' => Some(Key::M),
            'N' => Some(Key::N),
            'O' => Some(Key::O),
            'P' => Some(Key::P),
            'Q' => Some(Key::Q),
            'R' => Some(Key::R),
            'S' => Some(Key::S),
            'T' => Some(Key::T),
            'U' => Some(Key::U),
            'V' => Some(Key::V),
            'W' => Some(Key::W),
            'X' => Some(Key::X),
            'Y' => Some(Key::Y),
            'Z' => Some(Key::Z),
            '0' => Some(Key::Key0),
            '1' => Some(Key::Key1),
            '2' => Some(Key::Key2),
            '3' => Some(Key::Key3),
            '4' => Some(Key::Key4),
            '5' => Some(Key::Key5),
            '6' => Some(Key::Key6),
            '7' => Some(Key::Key7),
            '8' => Some(Key::Key8),
            '9' => Some(Key::Key9),
            _ => None,
        };
    }
    match s.to_ascii_uppercase().as_str() {
        "UP" | "ARROWUP" => Some(Key::Up),
        "DOWN" | "ARROWDOWN" => Some(Key::Down),
        "LEFT" | "ARROWLEFT" => Some(Key::Left),
        "RIGHT" | "ARROWRIGHT" => Some(Key::Right),
        "SPACE" | " " => Some(Key::Space),
        "ENTER" | "RETURN" => Some(Key::Enter),
        "ESC" | "ESCAPE" => Some(Key::Escape),
        "SHIFT" => Some(Key::LeftShift),
        "CTRL" | "CONTROL" => Some(Key::LeftCtrl),
        "TAB" => Some(Key::Tab),
        _ => None,
    }
}
