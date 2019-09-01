use crate::Config;
use crate::TermBuffer;
use crossterm::{self as ct, style, InputEvent, KeyEvent};
use std::fs::read_to_string;

#[derive(Debug)]
pub struct ScopePrompt<'a> {
    config: &'a mut Config,
    input: String,
    selected_index: u16,
    ty: &'a str,
    x_offset: u16,
}

pub enum ScopePromptResult {
    Scope(Option<String>),
    Escape,
    Terminate,
}

impl<'a> ScopePrompt<'a> {
    pub fn new(config: &'a mut Config, ty: &'a str) -> Self {
        ScopePrompt {
            config,
            input: Default::default(),
            selected_index: 0,
            ty,
            x_offset: 0,
        }
    }

    pub fn run(mut self) -> ScopePromptResult {
        let mut buffer = TermBuffer::new();

        let figlet_src = read_to_string("src/big.flf").expect("src/big.flf must exist");
        let font = crate::figlet::parse(figlet_src.lines()).expect("should be able to parse font");

        let input = crossterm::input();
        let mut sync_stdin = input.read_sync();

        let mut first_iteration = true;

        loop {
            let event = match first_iteration {
                true => {
                    first_iteration = false;
                    None
                }
                false => sync_stdin.next(),
            };

            match event {
                Some(InputEvent::Keyboard(KeyEvent::Ctrl('c'))) => {
                    return ScopePromptResult::Terminate;
                }
                Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                    return ScopePromptResult::Scope(Some(self.input).filter(|s| !s.is_empty()));
                }
                Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
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
                Some(InputEvent::Keyboard(KeyEvent::Left)) => {
                    self.x_offset = self.x_offset.saturating_sub(1);
                }
                Some(InputEvent::Keyboard(KeyEvent::Right)) => {
                    if (self.x_offset as usize) < self.input.len() {
                        self.x_offset += 1;
                    }
                }
                Some(InputEvent::Keyboard(KeyEvent::Backspace)) => {
                    let offset = self.x_offset as usize;
                    let len = self.input.len();
                    if offset < len - 1 {
                        self.input.remove(offset - 1);
                        self.x_offset -= 1;
                    } else if len > 0 {
                        self.input.pop();
                        self.x_offset -= 1;
                    }
                }
                Some(InputEvent::Keyboard(KeyEvent::Esc)) => {
                    return ScopePromptResult::Escape;
                }
                _ => {}
            };

            let mut lines = font.create_vec();

            let mut cursor_x = 0;
            cursor_x += font.write_to_buf_color(&self.ty, &mut lines[..], |s| {
                ct::style(s).with(ct::Color::Blue).to_string()
            });
            cursor_x += font.write_to_buf_color("(", &mut lines[..], |s| {
                ct::style(s).with(ct::Color::Grey).to_string()
            });

            let offset = self.x_offset as usize;
            cursor_x +=
                font.write_to_buf_color(&(self.input.as_str())[0..offset], &mut lines[..], |s| {
                    ct::style(s).with(ct::Color::Green).to_string()
                });

            // Insert the indicator for where input will be placed.
            // Note that
            font.write_to_buf_color("-", &mut lines[..], |s| {
                ct::style(s).with(ct::Color::Grey).to_string()
            });

            font.write_to_buf_color(&(self.input.as_str())[offset..], &mut lines[..], |s| {
                ct::style(s).with(ct::Color::Green).to_string()
            });
            font.write_to_buf_color(")", &mut lines[..], |s| {
                ct::style(s).with(ct::Color::Grey).to_string()
            });

            for line in lines {
                buffer.push_line(line);
            }

            buffer.set_next_cursor((cursor_x as u16, 3));
            buffer.render_frame();
            buffer.flush();
        }
    }
}
