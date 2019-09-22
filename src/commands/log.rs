use crate::cli;
use glint::{prompt, Commit, Config, Git};

fn with_raw<R>(f: impl FnOnce(crossterm::RawScreen) -> R) -> R {
    match crossterm::RawScreen::into_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(raw_screen) => f(raw_screen),
    }
}

pub fn log(params: cli::Log, mut config: Config) {
    let git = match Git::from_cwd() {
        Ok(git) => git,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let logs = git.log_parsed(params.git_args.iter()).expect("parse logs");

    for log in logs {
        println!("Log: {:?}", log);
    }
}
