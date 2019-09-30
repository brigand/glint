use crate::Figlet;
use std::io;

#[derive(Debug, Clone)]
pub struct Config {
    pub types: Vec<String>,
    pub figlet_file: Option<String>,
}

impl Config {
    pub fn get_figlet(&self) -> Result<Figlet, io::Error> {
        match self.figlet_file {
            Some(ref figlet_file) => Figlet::from_file(figlet_file),
            None => Ok(Figlet::default()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            types: vec![
                "build",
                "ci",
                "chore",
                "docs",
                "feat",
                "fix",
                "perf",
                "refactor",
                "revert",
                "style",
                "test",
                "enhance",
                "improvement",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
            figlet_file: None,
        }
    }
}
