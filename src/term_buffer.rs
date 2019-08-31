use crossterm as ct;
use std::io::{self, Write as _W};

/// A stateful view of what's currently rendered, and functionality
/// for updating it in a consistent way.
pub struct TermBuffer {
    state: State,
    flushed: State,

    // Cache some structs
    terminal: ct::Terminal,
    stdout: io::Stdout,
}

impl Drop for TermBuffer {
    fn drop(&mut self) {
        // self.cursor_to_end();
        self.state = Default::default();

        self.clear_and_render();
        ct::queue!(self.stdout, ct::Output("\n".to_string()));
        self.flush();
    }
}

impl TermBuffer {
    pub fn new() -> Self {
        TermBuffer {
            state: Default::default(),
            flushed: Default::default(),

            // Cache some structs
            terminal: ct::terminal(),
            stdout: io::stdout(),
        }
    }

    /// Add a row to the desired output
    pub fn push_line(&mut self, row: impl Into<String>) {
        self.state.push(row);
    }

    /// Clears from the cursor position down
    fn queue_clear(&mut self) {
        ct::queue!(self.stdout, ct::Clear(ct::ClearType::FromCursorDown));
    }

    fn cursor_to_start(&mut self) {
        let (x, y) = self.flushed.cursor;

        // if x > 0 {
        ct::queue!(self.stdout, ct::Left(1000));
        // }
        if y > 0 {
            ct::queue!(self.stdout, ct::Up(y));
        }
    }

    /// Positions the cursor where (0, 0) is the first character printed by this program
    pub fn set_cursor(&mut self, cursor: (u16, u16)) {
        self.state.set_cursor(cursor);
    }

    pub fn cursor_to_end(&mut self) {
        let (w, h) = self.terminal.terminal_size();
        ct::queue!(self.stdout, ct::Goto(0, h));
    }

    pub fn clear_and_render(&mut self) {
        self.cursor_to_start();
        self.queue_clear();

        let state = self.state.reset();

        for item in state.rows.iter() {
            ct::queue!(
                self.stdout,
                ct::Output(item.to_string()),
                ct::Output("\n".to_string()),
                ct::Left(1000)
            );
        }

        let (cx, cy) = (0, state.len() as u16);
        let (dx, dy) = state.get_cursor();
        if dy < cy {
            ct::queue!(self.stdout, ct::Up(cy - dy));
        } else if dy > cy {
            ct::queue!(self.stdout, ct::Down(dy - cy));
        }
        if dx < cx {
            ct::queue!(self.stdout, ct::Left(cx - dx));
        } else if dx > cx {
            ct::queue!(self.stdout, ct::Right(dx - cx));
        }

        ct::queue!(self.stdout, crate::color::reset_item());

        self.flushed = state;
    }

    pub fn flush(&mut self) {
        let _r = self.stdout.flush();

        // let cursor_saved = ct::cursor().pos();
        // self.cursor_to_end();
        // let _r = self.stdout.flush();

        // let cursor_final = ct::cursor().pos();
        // self.flushed.first_row = cursor_final.1 - self.flushed.rows;

        // ct::queue!(self.stdout, ct::Goto(cursor_saved.0, cursor_saved.1));
        // let _r = self.stdout.flush();
    }

    fn scroll_down(&mut self, count: i16) {
        if count < 1 {
            return;
        }

        // This happens immediately (I think) so we should update `flushed` now.
        self.flushed.first_row = self.flushed.first_row.saturating_sub(count as u16);
        self.state.first_row = self.state.first_row.saturating_sub(count as u16);

        let _r = self.terminal.scroll_down(count);
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
    pub fn is_empty(&self) -> bool {
        self.rows.len() < 1
    }

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
