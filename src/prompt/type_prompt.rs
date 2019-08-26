use crate::Config;
use crate::TermBuffer;
use crossterm::{cursor, queue, style, Clear, ClearType, Goto, InputEvent, KeyEvent, Output};
use std::io::{stdout, Write};

#[derive(Debug)]
pub struct TypePrompt<'a> {
    config: &'a mut Config,
    input: String,
    rows: Option<(u16, u16)>,
    selected_index: u16,
}

pub enum TypePromptResult {
    Type(String),
    Escape,
    Terminate,
}

impl<'a> TypePrompt<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        TypePrompt {
            config,
            input: Default::default(),
            rows: Default::default(),
            selected_index: 0,
        }
    }

    /// Attempts to find the item at `self.selected_index`. If greater than
    /// the number of items, then the last item, or finaly falling back to "misc" which
    /// doesn't normally appear in commitlint.
    fn get_at_selected_index(&self) -> &str {
        let options = self.filter_types();
        options
            .get(self.selected_index as usize)
            .or_else(|| options.last())
            .map(|&x| x)
            .unwrap_or("misc")
    }

    fn filter_types(&self) -> Vec<&str> {
        self.config
            .types
            .iter()
            .map(|x| x.as_str())
            .filter(|item| {
                if self.input.is_empty() {
                    true
                } else {
                    item.starts_with(&self.input)
                }
            })
            .collect()
    }

    pub fn run(mut self) -> TypePromptResult {
        let mut stdout = stdout();
        let cursor = cursor();
        let term = crossterm::terminal();

        let mut row: u16 = cursor.pos().1 + 1;

        if self.rows.is_none() {
            self.rows = Some((row, row));
        }

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

            let rows = self.rows.unwrap();

            eprintln!("{:?}", event);
            match event {
                Some(InputEvent::Keyboard(KeyEvent::Ctrl('c'))) => {
                    return TypePromptResult::Terminate;
                }
                Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                    return TypePromptResult::Type(self.get_at_selected_index().to_string());
                }
                Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
                    self.input.push(c.to_ascii_lowercase());
                }
                Some(InputEvent::Backspace(KeyEvent::Backspace)) => {
                    self.input.pop();
                }
                Some(InputEvent::Keyboard(KeyEvent::Esc)) => {
                    return TypePromptResult::Escape;
                }
                Some(InputEvent::Keyboard(KeyEvent::Up)) => {
                    self.selected_index = match self.selected_index {
                        0 => 0,
                        x => x + 1,
                    };
                }
                Some(InputEvent::Keyboard(KeyEvent::Down)) => {
                    self.selected_index += 1;
                }
                _ => {}
            };

            let types = self.filter_types();
            if types.len() == 1 {
                return TypePromptResult::Type(types[0].to_string());
            }

            let cursor_rest = {
                let prompt_pre = "Filter: ";
                let prompt_post = &self.input;
                queue!(
                    stdout,
                    Goto(0, row),
                    Output(format!(
                        "{}{} {}\n",
                        prompt_pre,
                        style(prompt_post).with(crate::color::theme_user_input()),
                        self.input.len()
                    ))
                );
                let x = prompt_pre.len() + prompt_post.len();
                let y = row;
                row += 1;
                (x as u16, y)
            };

            for ty in types {
                let color = crate::color::clint_type_to_color(ty);
                queue!(
                    stdout,
                    Goto(0, row),
                    Output(style(ty.to_string()).with(color).to_string()),
                    Output("\n".to_string())
                );
                row += 1;
                term.scroll_down(1);
            }
            row -= 1;

            if row < rows.1 {
                queue!(stdout, Goto(0, row + 1), Clear(ClearType::FromCursorDown));
            }

            self.rows = Some((rows.0, row));

            queue!(stdout, Goto(cursor_rest.0, cursor_rest.1));

            stdout.flush();
        }

        TypePromptResult::Escape
    }
}
