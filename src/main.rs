mod cli;
mod commands;

use cli::Cli;
use glint::Config;

fn main() {
    let command = cli::parse();
    let config = Config::default();

    match command {
        Cli::Commit(params) => {
            commands::commit(params, config);
        }
    }
}
