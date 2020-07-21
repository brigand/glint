use crate::string::{self, next_word_grapheme, prev_word_grapheme, to_byte_offset, to_byte_range};
use crate::Config;
use crate::TermBuffer;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{style, Color},
};

#[derive(Debug)]
pub struct MessagePrompt<'a> {
    config: &'a Config,
    input: Vec<String>,
    cursor: (u16, u16),
}

pub enum MessagePromptResult {
    Message(String),
    Escape,
    Terminate,
}

impl<'a> MessagePrompt<'a> {
    pub fn new(config: &'a Config) -> Self {
        MessagePrompt {
            config,
            input: vec![String::new()],
            cursor: (0, 0),
        }
    }

    pub fn run(mut self) -> MessagePromptResult {
        let mut buffer = TermBuffer::new();

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
                    return MessagePromptResult::Terminate;
                }
                Some((KeyCode::Char('a'), true, false, false)) => {
                    self.cursor.0 = 0;
                }
                Some((KeyCode::Char('e'), true, false, false)) => {
                    let (_, y) = self.cursor;
                    let line = self
                        .input
                        .get(y as usize)
                        .expect("ctrl-e unable to find current line");
                    self.cursor.0 = string::len(&line) as u16;
                }
                Some((KeyCode::Char('\n'), _, false, true))
                | Some((KeyCode::Char('\n'), true, false, _)) => {
                    self.input.push(String::new());
                    self.cursor.1 += 1;
                }
                Some((KeyCode::Enter, _, _, _)) => {
                    return MessagePromptResult::Message(self.input.join("\n"));
                }
                Some((KeyCode::Char(c), false, _, false)) if c > '\x1F' => {
                    let (x, y) = self.cursor;
                    let line = self.input.get_mut(y as usize).unwrap();
                    line.insert(to_byte_offset(&line, x as usize), c);
                    self.cursor.0 += 1;
                }
                Some((KeyCode::Left, false, _, false)) => {
                    self.cursor.0 = self.cursor.0.saturating_sub(1);
                }
                Some((KeyCode::Right, false, _, false)) => {
                    let (x, y) = self.cursor;
                    let line = self.input.get_mut(y as usize).expect("KE::Right get_mut");
                    if string::len(line) < x as usize + 1 {
                        line.push(' ');
                    }
                    self.cursor.0 += 1;
                }
                Some((KeyCode::Left, false, _, true))
                | Some((KeyCode::Char('b'), false, _, true)) => {
                    let (x, y) = self.cursor;
                    let line = &self.input.get(y as usize).expect("current line must exist");
                    self.cursor.0 = prev_word_grapheme(line, x as usize) as u16;
                }
                Some((KeyCode::Right, false, _, true))
                | Some((KeyCode::Char('f'), false, _, true)) => {
                    let (x, y) = self.cursor;
                    let line = &self.input.get(y as usize).expect("current line must exist");
                    self.cursor.0 = next_word_grapheme(line, x as usize) as u16;
                }
                Some((KeyCode::Up, false, _, _)) => {
                    self.cursor.1 = self.cursor.1.saturating_sub(1);
                }
                Some((KeyCode::Down, false, _, _)) => {
                    let (_, y) = self.cursor;
                    if (y as usize) + 1 >= self.input.len() {
                        self.input.push(String::new());
                    }
                    self.cursor = (0, y + 1);
                }
                // Alt-Backspace deletes a word.
                Some((KeyCode::Backspace, false, _, true))
                | Some((KeyCode::Char('\u{7f}'), false, _, true)) => match self.cursor {
                    (0, 0) => {}
                    (0, y) => {
                        self.input.remove(y as usize);
                        self.cursor.1 -= 1;
                        let line = &self.input[self.cursor.1 as usize];
                        self.cursor.0 = string::len(line) as u16;
                    }
                    (x, y) => {
                        let line = self
                            .input
                            .get_mut(y as usize)
                            .expect("Alt-Backspace (x, y) get line y");
                        let end = to_byte_offset(line, x as usize + 1);
                        let start = to_byte_offset(line, prev_word_grapheme(line, x as usize));
                        line.replace_range(start..end, "");

                        self.cursor.0 = string::len(&line[..start]) as u16;
                    }
                },
                Some((KeyCode::Backspace, false, _, false)) => match self.cursor {
                    (0, 0) => {}
                    (0, y) => {
                        self.input.remove(y as usize);
                        self.cursor.1 -= 1;
                        let line = &self.input[self.cursor.1 as usize];
                        self.cursor.0 = string::len(line) as u16;
                    }
                    (x, y) => {
                        let line = self
                            .input
                            .get_mut(y as usize)
                            .expect("Backspace (x, y) get line y");

                        if x as usize >= string::len(line) {
                            line.pop();
                        } else {
                            line.replace_range(to_byte_range(line as &str, x as usize - 1), "");
                        }
                        self.cursor.0 -= 1;
                    }
                },
                Some((KeyCode::Char('d'), true, _, false)) => {
                    let line = &mut self.input[self.cursor.1 as usize];

                    line.replace_range(to_byte_range(line, self.cursor.0 as usize), "");
                }
                Some((KeyCode::Esc, false, _, false)) => {
                    return MessagePromptResult::Escape;
                }
                None => {}
                _ => continue,
            };

            let (x, y) = self.cursor;
            let instructions = "Commit message (arrow keys for multiple lines):";
            let divider = "-".repeat(instructions.len());
            buffer.push_line(instructions);
            buffer.push_line(divider);

            // The offset for where the editor begins, i.e. the number of push_line calls above.
            let editor_y = 2;

            for (i, line) in self.input.iter().enumerate() {
                if i == 0 && line.len() > 50 {
                    let (good, bad) = crate::string::split_at(&line, 50);
                    buffer.push_line(format!(
                        "{}{}{}",
                        good,
                        style(bad).with(Color::Red),
                        crate::color::reset_display(),
                    ));
                } else {
                    buffer.push_line(line.to_string());
                }
            }

            buffer.set_next_cursor((x, y + editor_y));
            buffer.render_frame();
            buffer.flush();
        }
    }
}
