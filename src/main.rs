mod cli;
use clap::ArgMatches;
use clint::{prompt, Commit, Config};

fn with_raw<R>(f: impl FnOnce(crossterm::RawScreen) -> R) -> R {
    match crossterm::RawScreen::into_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(raw_screen) => f(raw_screen),
    }
}

fn commit(args: ArgMatches, mut config: Config) {
    enum Stage {
        Type,
        Scope(String),
        Message(String, Option<String>),
        Complete(String, Option<String>, String),
    }

    let mut stage = Stage::Type;

    loop {
        match stage {
            Stage::Type => {
                let ty = match args.value_of("TYPE") {
                    Some(ty) => Some(ty.to_string()),
                    None => with_raw(|_raw| match prompt::TypePrompt::new(&mut config).run() {
                        prompt::TypePromptResult::Type(ty) => Some(ty),
                        prompt::TypePromptResult::Terminate => None,
                        prompt::TypePromptResult::Escape => None,
                    }),
                };

                let ty = match ty {
                    Some(s) => s,
                    None => std::process::exit(1),
                };

                stage = Stage::Scope(ty);
            }
            Stage::Scope(ty) => {
                let mut escape = false;
                let scope =
                    match args.value_of("SCOPE") {
                        Some(scope) => Some(Some(scope.to_string())),
                        None => with_raw(|_raw| {
                            match prompt::ScopePrompt::new(&mut config, &ty).run() {
                                prompt::ScopePromptResult::Scope(scope) => Some(scope),
                                prompt::ScopePromptResult::Terminate => None,
                                prompt::ScopePromptResult::Escape => {
                                    escape = true;
                                    None
                                }
                            }
                        }),
                    };

                if escape {
                    stage = Stage::Type;
                    continue;
                }

                let scope = match scope {
                    Some(s) => s,
                    None => std::process::exit(1),
                };

                stage = Stage::Message(ty, scope);
            }
            Stage::Message(ty, scope) => {
                let mut escape = false;
                let message = match args.value_of("MESSAGE") {
                    Some(message) => Some(message.to_string()),
                    None => with_raw(|_raw| match prompt::MessagePrompt::new(&mut config).run() {
                        prompt::MessagePromptResult::Message(message) => Some(message),
                        prompt::MessagePromptResult::Terminate => None,
                        prompt::MessagePromptResult::Escape => {
                            escape = true;
                            None
                        }
                    }),
                };

                if escape {
                    stage = Stage::Scope(ty);

                    continue;
                }

                let message = match message {
                    Some(s) => s,
                    None => std::process::exit(1),
                };

                stage = Stage::Complete(ty, scope, message);
            }
            Stage::Complete(ty, scope, message) => {
                let commit = Commit { ty, scope, message };

                println!("Commit:\n{}", commit.build_message());
                return;
            }
        }
    }
}

fn main() {
    let args = cli::parse();
    let config = Config::default();

    commit(args, config);
}
