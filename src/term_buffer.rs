use crossterm as ct;
use std::io::{self, Write as _W};

// If the number of changed lines is larger than this, then
// we do a full paint.
const MAX_PATCH_LINES: usize = 3;

/// Represents a range of lines in a terminal and the cursor position. This is
/// suitable when you don't want to use an "alternate screen", but rather retain
/// previous terminal output, such as shell prompts/responses.
///
/// New frames are rendered by replacing the lines. All operations work on a relative
/// coordinate system where (0, 0) is the top-left corner of the lines TermBuffer controls.
///
/// Further, we never check the actual cursor position, but rather move the cursor relative
/// to its current position. The meaning of (0, 0) is actually the cursor position when TermBuffer
/// first renders.
pub struct TermBuffer {
    state: State,
    flushed: State,

    // Cache some structs
    // terminal: ct::Terminal,
    stdout: io::Stderr,
}

impl Drop for TermBuffer {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            self.state = Default::default();
            self.render_frame();
        }
        self.cursor_to_end();

        ct::queue!(self.stdout, ct::Output("\n".to_string())).unwrap();
        self.flush();
    }
}

impl Default for TermBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TermBuffer {
    pub fn new() -> Self {
        TermBuffer {
            state: Default::default(),
            flushed: Default::default(),

            // Cache some structs
            // terminal: ct::terminal(),
            stdout: io::stderr(),
        }
    }

    /// Add a row to the desired output
    pub fn push_line(&mut self, row: impl Into<String>) {
        self.state.push(row);
    }

    pub fn lines(&self) -> u16 {
        self.state.len() as u16
    }

    /// Positions the cursor where (0, 0) is the first character printed by this program
    pub fn set_next_cursor(&mut self, cursor: (u16, u16)) {
        self.state.set_cursor(cursor);
    }

    /// This causes us to skip past the currently displayed buffer area and forget about it,
    /// resulting in future renders to happen below it.
    /// If this is called, and then the TermBuffer is dropped, the default behavior of clearing
    /// the area will be a no-op.
    pub fn forget(&mut self) -> usize {
        let lines = self.flushed.len();

        self.cursor_to_end();
        self.state = Default::default();
        self.flushed = Default::default();

        lines
    }

    /// Perform the necessary update to the terminal. This may choose a more
    /// optimized update than a full frame.
    pub fn render_frame(&mut self) {
        let same_line_count = self.state.len() == self.flushed.len();

        if !same_line_count {
            return self.render_full();
        }

        let changed_lines: Vec<_> = self
            .state
            .iter()
            .zip(self.flushed.iter())
            .enumerate()
            .filter_map(|(i, (a, b))| if a == b { None } else { Some(i) })
            .collect();

        if !changed_lines.is_empty() && changed_lines.len() <= MAX_PATCH_LINES {
            for line_num in changed_lines {
                self.render_one_line(line_num);
            }
            self.flushed = self.state.reset();
        } else {
            self.render_full();
        }
    }

    fn queue_move_cursor_y(&mut self, down: isize) {
        if down > 0 {
            let down = down as u16;
            ct::queue!(self.stdout, ct::Down(down), ct::Left(1000)).unwrap();
        } else if down < 0 {
            let up = (-down) as u16;
            ct::queue!(self.stdout, ct::Up(up), ct::Left(1000)).unwrap();
        } else {
            ct::queue!(self.stdout, ct::Left(1000)).unwrap();
        }
    }

    pub fn render_one_line(&mut self, line_index: usize) {
        let down = line_index as isize - self.flushed.cursor.1 as isize;

        let state = self.state.clone();

        self.queue_move_cursor_y(down);
        let new_y = (self.flushed.cursor.1 as isize + down) as u16;

        let (dx, dy) = state.cursor;

        ct::queue!(self.stdout, ct::Clear(ct::ClearType::UntilNewLine)).unwrap();

        ct::queue!(self.stdout, ct::Output(state.rows[line_index].to_string())).unwrap();

        // This can be enabled to track which lines are updated
        // ct::queue!(self.stdout, ct::Output(" ø".to_string())).unwrap();

        ct::queue!(self.stdout, ct::Left(1000)).unwrap();

        self.queue_move_cursor_y(dy as isize - new_y as isize);
        if dx > 0 {
            ct::queue!(self.stdout, ct::Right(dx)).unwrap();
        }
        self.flushed.cursor = (0, dy);
    }

    /// Renders a complete frame to the terminal
    pub fn render_full(&mut self) {
        self.cursor_to_start();
        self.queue_clear();

        let state = self.state.reset();

        for item in state.rows.iter() {
            ct::queue!(
                self.stdout,
                ct::Output(item.to_string()),
                ct::Output("\n".to_string()),
                ct::Left(1000)
            )
            .unwrap();
        }

        let (cx, cy) = (0, state.len() as u16);
        let (dx, dy) = state.get_cursor();
        if dy < cy {
            ct::queue!(self.stdout, ct::Up(cy - dy)).unwrap();
        } else if dy > cy {
            ct::queue!(self.stdout, ct::Down(dy - cy)).unwrap();
        }
        if dx < cx {
            ct::queue!(self.stdout, ct::Left(cx - dx)).unwrap();
        } else if dx > cx {
            ct::queue!(self.stdout, ct::Right(dx - cx)).unwrap();
        }

        ct::queue!(self.stdout, crate::color::reset_item()).unwrap();

        self.flushed = state;
    }

    pub fn flush(&mut self) {
        self.stdout.flush().expect("flush failed");
    }

    fn cursor_to_end(&mut self) {
        let (cursor_x, cursor_y) = self.flushed.get_cursor();
        let height = self.flushed.len() as u16;
        let down = height.saturating_sub(cursor_y);

        let move_down = down > 0;
        let move_left = cursor_x > 0;
        if move_down {
            ct::queue!(self.stdout, ct::Down(down)).unwrap();
        }
        if move_left {
            ct::queue!(self.stdout, ct::Left(cursor_x)).unwrap();
        }

        if move_down || move_left {
            self.flush();
        }
    }

    /// Clears from the cursor position down
    fn queue_clear(&mut self) {
        ct::queue!(self.stdout, ct::Clear(ct::ClearType::FromCursorDown)).unwrap();
    }

    fn cursor_to_start(&mut self) {
        let (_, y) = self.flushed.cursor;

        // if x > 0 {
        ct::queue!(self.stdout, ct::Left(1000)).unwrap();
        // }
        if y > 0 {
            ct::queue!(self.stdout, ct::Up(y)).unwrap();
        }
    }
}

/// Represents internal state of TermBuffer
#[derive(Clone, Debug)]
struct State {
    cursor: (u16, u16),
    rows: Vec<String>,
    first_row: u16,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cursor == other.cursor && self.rows == other.rows
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            cursor: (0, 0),
            rows: vec![],
            first_row: 0,
        }
    }
}

impl State {
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn push(&mut self, row: impl Into<String>) {
        self.rows.push(row.into());
    }

    pub fn set_cursor(&mut self, cursor: (u16, u16)) {
        self.cursor = cursor;
    }

    pub fn get_cursor(&self) -> (u16, u16) {
        self.cursor
    }

    pub fn reset(&mut self) -> Self {
        std::mem::replace(self, State::default())
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.rows.iter().map(|s| s.as_str())
    }
}
