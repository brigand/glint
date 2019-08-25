#[derive(Debug, Clone)]
pub struct Config {
    pub types: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            types: vec![
                "build", "ci", "chore", "docs", "feat", "fix", "perf", "refactor", "revert",
                "style", "test",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }
}
