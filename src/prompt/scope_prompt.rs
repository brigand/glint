use crate::Config;
use crate::TermBuffer;
use crossterm::{
    self as ct,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{style, Color},
};

#[derive(Debug)]
pub struct ScopePrompt<'a> {
    config: &'a Config,
    input: String,
    selected_index: u16,
    ty: &'a str,
    x_offset: u16,
    finished: bool,
}

pub enum ScopePromptResult {
    Scope(Option<String>, usize),
    Escape,
    Terminate,
}

impl<'a> ScopePrompt<'a> {
    pub fn new(config: &'a Config, ty: &'a str) -> Self {
        ScopePrompt {
            config,
            input: Default::default(),
            selected_index: 0,
            ty,
            x_offset: 0,
            finished: false,
        }
    }

    pub fn run(mut self) -> ScopePromptResult {
        let mut buffer = TermBuffer::new();

        let figlet = self
            .config
            .get_figlet()
            .expect("Ensure figlet_file points to a valid file, or remove it.");

        let mut first_iteration = true;

        loop {
            let event = if first_iteration {
                first_iteration = false;
                None
            } else {
                match event::read() {
                    Ok(Event::Key(KeyEvent { code, modifiers })) => Some((
                        code,
                        modifiers.contains(KeyModifiers::CONTROL),
                        modifiers.contains(KeyModifiers::SHIFT),
                        modifiers.contains(KeyModifiers::ALT),
                    )),
                    _ => continue,
                }
            };

            match event {
                Some((KeyCode::Char('c'), true, false, false)) => {
                    return ScopePromptResult::Terminate;
                }
                Some((KeyCode::Enter, false, false, false)) => {
                    self.finished = true;
                }
                Some((KeyCode::Char(c), false, _, false)) => {
                    let accept = (c >= 'a' && c <= 'z')
                        || (c >= 'A' && c <= 'Z')
                        || (c >= '0' && c <= '9')
                        || (c == '_')
                        || c == '-'
                        || c == '/'
                        || c == ','
                        || c == '|';
                    if accept {
                        self.x_offset += 1;

                        self.input
                            .insert(self.x_offset as usize - 1, c.to_ascii_lowercase());
                    }
                }
                Some((KeyCode::Left, false, _, false)) => {
                    self.x_offset = self.x_offset.saturating_sub(1);
                }
                Some((KeyCode::Right, false, _, false)) => {
                    if (self.x_offset as usize) < self.input.len() {
                        self.x_offset += 1;
                    }
                }
                Some((KeyCode::Backspace, false, _, false)) => {
                    let offset = self.x_offset as usize;
                    let len = self.input.len();
                    if len > 0 && offset < len - 1 {
                        self.input.remove(offset - 1);
                        self.x_offset -= 1;
                    } else if len > 0 {
                        self.input.pop();
                        self.x_offset -= 1;
                    }
                }
                Some((KeyCode::Esc, false, _, false)) => {
                    return ScopePromptResult::Escape;
                }
                None => {}
                _ => continue,
            };

            let (term_width, _) = ct::terminal::size().expect("get terminal size");

            let mut lines = figlet.create_vec();

            let mut cursor_x = 0;
            cursor_x += figlet.write_to_buf_color(&self.ty, &mut lines[..], |s| {
                style(s).with(Color::Blue).to_string()
            });

            let show_parens = !self.finished || !self.input.is_empty();

            if show_parens {
                cursor_x += figlet.write_to_buf_color("(", &mut lines[..], |s| {
                    style(s).with(Color::Grey).to_string()
                });
            }

            let offset = self.x_offset as usize;
            cursor_x +=
                figlet.write_to_buf_color(&(self.input.as_str())[0..offset], &mut lines[..], |s| {
                    style(s).with(Color::Green).to_string()
                });

            let mut fig_width = cursor_x;

            // Insert the indicator for where input will be placed.
            // Note that
            if !self.finished {
                fig_width += figlet.write_to_buf_color("-", &mut lines[..], |s| {
                    style(s).with(Color::Grey).to_string()
                });
            }

            fig_width +=
                figlet.write_to_buf_color(&(self.input.as_str())[offset..], &mut lines[..], |s| {
                    style(s).with(Color::Green).to_string()
                });

            if show_parens {
                fig_width += figlet.write_to_buf_color(")", &mut lines[..], |s| {
                    style(s).with(Color::Grey).to_string()
                });
            }

            fig_width += figlet.write_to_buf_color(":", &mut lines[..], |s| {
                style(s).with(Color::Grey).to_string()
            });

            // We're tracking the printed width above to see if we've run out of space here.
            let figlet_overflows = fig_width + 1 > term_width as usize;

            let cursor_y = if figlet_overflows { 1 } else { 3 };

            // If we did overflow, then for now we should display it as a single line with one line of padding above/below
            if figlet_overflows {
                use std::fmt::Write;

                lines = vec!["".into(), "".into(), "".into()];
                let line = &mut lines[1];

                write!(line, "{}", style(&self.ty).with(Color::Blue)).unwrap();
                write!(line, "{}", style("(").with(Color::Grey)).unwrap();
                write!(
                    line,
                    "{}",
                    style(&(self.input.as_str())[0..offset]).with(Color::Green)
                )
                .unwrap();

                if !self.finished {
                    write!(line, "{}", style("_").with(Color::Grey)).unwrap();
                }
                write!(line, "{}", style(")").with(Color::Grey)).unwrap();

                cursor_x = self.ty.len() + 1 + self.input.len();
            }

            for line in lines {
                buffer.push_line(line);
            }

            buffer.set_next_cursor((cursor_x as u16, cursor_y));
            buffer.render_frame();
            buffer.flush();

            if self.finished {
                let rows = buffer.forget();
                return ScopePromptResult::Scope(Some(self.input).filter(|s| !s.is_empty()), rows);
            }
        }
    }
}
