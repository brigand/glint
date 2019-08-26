use crossterm as ct;
use std::io;

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
    }

    pub fn push_row(&mut self, row: impl Into<String>) {
        self.rows.push(row);
        self.state.rows += 1;
    }

    pub fn clear(&mut self) {
        ct::queue!(
            self.stdout,
            ct::Goto(0, self.flushed.first_row),
            ct::Clear(ct::ClearType::FromCursorDown)
        );
        self.rows.empty();
    }

    pub fn flush(&mut self) {
        self.stdout.flush();
    }

    fn scroll_down(&mut self, count: i16) {
        // This happens immediately (I think) so we should update `flushed` now.
        self.flushed.first_row = self.flushed.first_row.saturating_sub(count);

        self.state.first_row = self.state.first_row.saturating_sub(count);
        self.terminal.scroll_down(count);
    }
}

/// Represents internal state of TermBuffer
struct State {
    cursor: (u16, u16),
    rows: u16,
    first_row: u16,
}

impl Default for State {
    fn default() -> Self {
        State {
            cursor: (u16, u16),
            rows: 0,
            first_row: 0,
        }
    }
}
