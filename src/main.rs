mod cli;

use cli::Cli;
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

fn commit(params: cli::Commit, mut config: Config) {
    let git = match Git::from_cwd() {
        Ok(git) => git,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    println!("{:?}", git.status());
    return;

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
                let ty = match params.ty {
                    Some(ref ty) => Some(ty.to_string()),
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
                    match params.scope {
                        Some(ref scope) => Some(Some(scope.to_string())),
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
                let message = match params.message {
                    Some(ref message) => Some(message.to_string()),
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

                let git_message = commit.build_message();
                match git.commit(&git_message, params.git_args).status() {
                    Ok(status) if status.success() => println!("Commit successful."),
                    Ok(status) => match status.code() {
                        Some(code) => {
                            eprintln!("Commit command failed with {}", code);
                            std::process::exit(code);
                        }
                        None => {
                            eprintln!("Commit command failed with no status. Was likely killed by another process.");
                            std::process::exit(1);
                        }
                    },
                    Err(err) => {
                        eprintln!(
                            "Failed to run git. This is the best error I have:\n{:?}",
                            err
                        );
                        std::process::exit(1);
                    }
                };

                return;
            }
        }
    }
}

fn main() {
    let command = cli::parse();
    let config = Config::default();

    match command {
        Cli::Commit(params) => {
            commit(params, config);
        }
    }
}
