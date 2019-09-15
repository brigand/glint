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
    //x
}

fn commit(params: cli::Commit, mut config: Config) {
    let git = match Git::from_cwd() {
        Ok(git) => git,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    enum Stage {
        Files,
        Type,
        Scope(String),
        Message(String, Option<String>),
        Complete(String, Option<String>, String),
    }

    let mut stage = Stage::Type;

    let git_status = git.status().ok();

    if let Some(ref git_status) = git_status {
        let any_staged = git_status.any_staged();
        let any_unstaged = git_status.any_unstaged();

        if !any_staged && any_unstaged {
            if params.git_args.is_empty() {
                stage = Stage::Files;
            }
        } else if !any_staged {
            eprintln!("No changes to commit.");
            std::process::exit(1);
        }
    }

    let mut commit_files: Option<Vec<String>> = None;

    loop {
        match stage {
            Stage::Files => {
                commit_files = with_raw(|_raw| {
                    match prompt::FilesPrompt::new(&mut config, &git, git_status.clone().unwrap())
                        .run()
                    {
                        prompt::FilesPromptResult::Files(files) => Some(files),
                        prompt::FilesPromptResult::Terminate => None,
                        prompt::FilesPromptResult::Escape => None,
                    }
                });

                if commit_files.is_none() {
                    std::process::exit(1);
                }

                stage = Stage::Type;
            }
            Stage::Type => {
                let mut escape = false;

                let ty = match params.ty {
                    Some(ref ty) => Some(ty.to_string()),
                    None => with_raw(|_raw| match prompt::TypePrompt::new(&mut config).run() {
                        prompt::TypePromptResult::Type(ty) => Some(ty),
                        prompt::TypePromptResult::Terminate => None,
                        prompt::TypePromptResult::Escape => {
                            escape = true;
                            None
                        }
                    }),
                };

                if escape && commit_files.is_some() {
                    stage = Stage::Files;
                    continue;
                }

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
                if let Some(commit_files) = commit_files {
                    let _r = git.add(commit_files).status();
                }

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
