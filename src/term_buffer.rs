use crossterm as ct;
use std::io::{self, Write as _W};

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
    stdout: io::Stdout,
}

impl Drop for TermBuffer {
    fn drop(&mut self) {
        // self.cursor_to_end();
        self.state = Default::default();

        self.render_frame();
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
            stdout: io::stdout(),
        }
    }

    /// Add a row to the desired output
    pub fn push_line(&mut self, row: impl Into<String>) {
        self.state.push(row);
    }

    /// Positions the cursor where (0, 0) is the first character printed by this program
    pub fn set_next_cursor(&mut self, cursor: (u16, u16)) {
        self.state.set_cursor(cursor);
    }

    fn cursor_to_end(&mut self) {
        let cursor_y = self.flushed.get_cursor().1;
        let height = self.flushed.len() as u16;
        let down = height.saturating_sub(cursor_y);
        if down > 0 {
            ct::queue!(self.stdout, ct::Down(down)).unwrap();
            self.flush();
        }
    }

    /// This causes us to skip past the currently displayed buffer area and forget about it,
    /// resulting in future renders to happen below it.
    /// If this is called, and then the TermBuffer is dropped, the default behavior of clearing
    /// the area will be a no-op.
    pub fn forget(&mut self) {
        self.cursor_to_end();
        self.state = Default::default();
        self.flushed = Default::default();
    }

    pub fn render_frame(&mut self) {
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
}
