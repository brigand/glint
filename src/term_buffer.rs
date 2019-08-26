use crossterm as ct;
use std::io::{self, Write as _W};

/// A stateful view of what's currently rendered, and functionality
/// for updating it in a consistent way.
pub struct TermBuffer {
    state: State,
    flushed: State,
    rows: Vec<String>,

    // Cache some structs
    terminal: ct::Terminal,
    stdout: io::Stdout,
}

impl Drop for TermBuffer {
    fn drop(&mut self) {
        self.cursor_to_end();
        ct::queue!(
            self.stdout,
            crate::color::reset_item(),
            ct::Output("\n".to_string())
        );
        self.stdout.flush();
    }
}

impl TermBuffer {
    pub fn new() -> Self {
        TermBuffer {
            state: Default::default(),
            flushed: Default::default(),
            rows: Default::default(),

            // Cache some structs
            terminal: ct::terminal(),
            stdout: io::stdout(),
        }
        .init()
    }

    /// Private utility to keep ::new() clean
    fn init(mut self) -> Self {
        let (x, y) = ct::cursor().pos();

        self.state.cursor = (x, y);
        self.state.first_row = y;
        self.flushed = self.state.clone();

        self
    }

    /// Add a row to the desired output
    pub fn push_line(&mut self, row: impl Into<String>) {
        self.rows.push(row.into());
        self.state.rows += 1;
    }

    fn queue_clear(&mut self) {
        ct::queue!(
            self.stdout,
            ct::Goto(0, self.flushed.first_row),
            ct::Clear(ct::ClearType::FromCursorDown)
        );
    }

    pub fn cursor_to_end(&mut self) {
        let (w, h) = self.terminal.terminal_size();
        ct::queue!(self.stdout, ct::Goto(0, h));
    }

    pub fn clear(&mut self) {
        self.queue_clear();
        self.rows.clear();
    }

    pub fn clear_and_render(&mut self) {
        let row_count = self.rows.len() as u16;
        let additional_rows = row_count.saturating_sub(self.flushed.rows);

        self.queue_clear();

        let blank_lines: String = std::iter::repeat("\n")
            .take(additional_rows as usize)
            .collect();

        self.scroll_down(additional_rows as i16);

        ct::queue!(
            self.stdout,
            ct::Goto(0, self.flushed.first_row),
            ct::Output(blank_lines),
            ct::Goto(0, self.flushed.first_row)
        );

        self.flushed.rows = row_count;

        let rows = std::mem::replace(&mut self.rows, Vec::new());

        for (i, item) in rows.into_iter().enumerate() {
            let y = self.flushed.first_row + i as u16;
            ct::queue!(self.stdout, ct::Goto(0, y), ct::Output(item));
        }
        ct::queue!(self.stdout, crate::color::reset_item());
    }

    pub fn set_cursor_relative_to_flush(&mut self, x: u16, y: u16) {
        ct::queue!(self.stdout, ct::Goto(x, y + self.flushed.first_row - 1));
        self.flushed.cursor = (x, y);
        let _r = self.stdout.flush();
    }

    pub fn flush(&mut self) {
        let _r = self.stdout.flush();

        let cursor_saved = ct::cursor().pos();
        self.cursor_to_end();
        let _r = self.stdout.flush();

        let cursor_final = ct::cursor().pos();
        self.flushed.first_row = cursor_final.1 - self.flushed.rows;

        ct::queue!(self.stdout, ct::Goto(cursor_saved.0, cursor_saved.1));
        let _r = self.stdout.flush();
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
    rows: u16,
    first_row: u16,
}

impl Default for State {
    fn default() -> Self {
        State {
            cursor: (0, 0),
            rows: 0,
            first_row: 0,
        }
    }
}
