use crate::Config;
use crossterm::{cursor, queue, style, Clear, ClearType, Colored, Command, Goto, Output};
use std::io::{stdout, Write};

#[derive(Debug)]
pub struct TypePrompt<'a> {
    config: &'a mut Config,
    input: String,
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
        }
    }

    fn filter_types(&self) -> impl Iterator<Item = &str> {
        self.config.types.iter().map(|x| x.as_str())
    }

    pub fn run(mut self) -> TypePromptResult {
        let mut stdout = stdout();
        let cursor = cursor();
        let mut row = cursor.pos().1 + 1;
        for ty in self.filter_types() {
            let color = crate::color::clint_type_to_color(ty);
            queue!(
                stdout,
                Goto(0, row),
                Output(style(ty.to_string()).with(color).to_string()),
                Output("\n".to_string())
            );
            row += 1;
            crossterm::terminal().scroll_down(1);
        }
        stdout.flush();
        std::thread::sleep_ms(10000);

        TypePromptResult::Escape
    }
}
