use crate::color::reset_display;
use crate::Config;
use crate::TermBuffer;
use crossterm::{self as ct, style, InputEvent, KeyEvent};

#[derive(Debug)]
pub struct TypePrompt<'a> {
    config: &'a Config,
    input: String,
    selected_index: u16,
}

pub enum TypePromptResult {
    Type(String),
    Escape,
    Terminate,
}

impl<'a> TypePrompt<'a> {
    pub fn new(config: &'a Config) -> Self {
        TypePrompt {
            config,
            input: Default::default(),
            selected_index: 0,
        }
    }

    /// Attempts to find the item at `self.selected_index`. If greater than
    /// the number of items, then the last item, or finally falling back to "misc" which
    /// doesn't normally appear in commitlint.
    fn get_at_selected_index(&self) -> &str {
        let options = self.filter_types();
        options
            .get(self.selected_index as usize)
            .or_else(|| options.last())
            .copied()
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
                match sync_stdin.next() {
                    Some(e) => Some(e),
                    _ => continue,
                }
            };

            match event {
                Some(InputEvent::Keyboard(KeyEvent::Ctrl('c'))) => {
                    return TypePromptResult::Terminate;
                }
                Some(InputEvent::Keyboard(KeyEvent::Enter)) => {
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
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
                Some(InputEvent::Keyboard(KeyEvent::Down)) => {
                    self.selected_index += 1;
                }
                None => {}
                _ => continue,
            };

            let types = self.filter_types();
            if types.len() == 1 {
                return TypePromptResult::Type(types[0].to_string());
            }

            let mut header = figlet.create_vec();
            figlet.write_to_buf_color("<glint>", header.as_mut_slice(), |s| {
                ct::style(s).with(ct::Color::Magenta).to_string()
            });

            let y_offset = header.len() as u16;

            for line in header {
                buffer.push_line(line);
            }

            let after_prompt_x = {
                let prompt_pre = "Choose a type: ";
                let prompt_post = &self.input;
                let underscores = "_".repeat(6 - self.input.len());
                buffer.push_line(format!(
                    "{}{}{}{}",
                    prompt_pre,
                    style(prompt_post).with(crate::color::theme_user_input()),
                    underscores,
                    reset_display()
                ));
                let x = prompt_pre.len() + prompt_post.len();
                x as u16
            };

            let active = style("*").with(ct::Color::Blue).to_string();
            for (i, ty) in types.into_iter().enumerate() {
                let prefix = if i as u16 == self.selected_index {
                    &active as &str
                } else {
                    "-"
                };

                let line = format!("{} {}{}", prefix, ty, reset_display());
                buffer.push_line(line);
            }

            buffer.set_next_cursor((after_prompt_x, y_offset));
            buffer.render_frame();
            buffer.flush();
        }
    }
}
