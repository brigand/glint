use crate::color::reset_display;
use crate::Config;
use crate::TermBuffer;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    style::{style, Color},
};

#[derive(Debug)]
pub struct TypePrompt<'a> {
    config: &'a Config,
    input: String,
    focused_index: u16,
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
            focused_index: 0,
        }
    }

    /// Attempts to find the item at `self.selected_index`. If greater than
    /// the number of items, then the last item, or finally falling back to "misc" which
    /// doesn't normally appear in commitlint.
    fn get_at_selected_index(&self) -> &str {
        let options = self.filter_types();
        options
            .get(self.focused_index as usize)
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
                    return TypePromptResult::Terminate;
                }
                Some((KeyCode::Enter, false, false, false)) => {
                    return TypePromptResult::Type(self.get_at_selected_index().to_string());
                }
                Some((KeyCode::Char(c), false, _, false)) => {
                    self.input.push(c.to_ascii_lowercase());
                }
                Some((KeyCode::Backspace, false, _, false)) => {
                    self.input.pop();
                }
                Some((KeyCode::Esc, false, _, false)) => {
                    return TypePromptResult::Escape;
                }
                Some((KeyCode::Up, false, _, false)) => {
                    self.focused_index = self.focused_index.saturating_sub(1);
                }
                Some((KeyCode::Down, false, _, false)) => {
                    self.focused_index += 1;
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
                style(s).with(Color::Magenta).to_string()
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
                    style(underscores).with(crate::color::theme_user_input()),
                    reset_display()
                ));
                let x = prompt_pre.len() + prompt_post.len();
                x as u16
            };

            let focused_color = Color::Blue;
            let default_color = Color::Reset;

            for (i, ty) in types.into_iter().enumerate() {
                let line_content = if i as u16 == self.focused_index {
                    style(["*", " ", ty].concat()).with(focused_color)
                } else {
                    style(["-", " ", ty].concat()).with(default_color)
                };

                let line = format!("{}{}", line_content, reset_display());
                buffer.push_line(line);
            }

            buffer.set_next_cursor((after_prompt_x, y_offset));
            buffer.render_frame();
            buffer.flush();
        }
    }
}
