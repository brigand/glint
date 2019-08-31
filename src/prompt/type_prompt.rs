use crate::Config;
use crate::TermBuffer;
use crossterm::{queue, style, Clear, ClearType, Goto, InputEvent, KeyEvent, Output};

#[derive(Debug)]
pub struct TypePrompt<'a> {
    config: &'a mut Config,
    input: String,
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
        let mut buffer = TermBuffer::new();

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
                    return TypePromptResult::Terminate;
                }
                Some(InputEvent::Keyboard(KeyEvent::Char('\n'))) => {
                    return TypePromptResult::Type(self.get_at_selected_index().to_string());
                }
                Some(InputEvent::Keyboard(KeyEvent::Char(c))) => {
                    self.input.push(c.to_ascii_lowercase());
                }
                Some(InputEvent::Keyboard(KeyEvent::Backspace)) => {
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

            let after_prompt_x = {
                let prompt_pre = "Filter: ";
                let prompt_post = &self.input;
                buffer.push_line(format!(
                    "{}{} {}",
                    prompt_pre,
                    style(prompt_post).with(crate::color::theme_user_input()),
                    self.input.len()
                ));
                let x = prompt_pre.len() + prompt_post.len();
                x as u16
            };

            for ty in types {
                let color = crate::color::clint_type_to_color(ty);
                let line = format!("{}", style(ty.to_string()).with(color),);
                buffer.push_line(line);
            }

            buffer.set_next_cursor((after_prompt_x, 0));
            buffer.render_frame();
            buffer.flush();
        }
    }
}
