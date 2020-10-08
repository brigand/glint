use crate::cli;
use crossterm::{self as ct, cursor, terminal};
use glint::{prompt, Commit, Config, Git};
use std::io::Write as _Write;

fn with_raw<R>(f: impl FnOnce() -> R) -> R {
    let result = match terminal::enable_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(_) => f(),
    };

    let _r = terminal::disable_raw_mode();

    result
}

pub fn commit(params: cli::Commit, config: Config) {
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

    let mut escape_clear_lines = 0;

    loop {
        match stage {
            Stage::Files => {
                commit_files = with_raw(|| {
                    match prompt::FilesPrompt::new(&config, &git, git_status.clone().unwrap()).run()
                    {
                        prompt::FilesPromptResult::Files(files) => Some(files),
                        prompt::FilesPromptResult::Terminate => std::process::exit(2),
                        prompt::FilesPromptResult::Escape => std::process::exit(0),
                    }
                });

                stage = Stage::Type;
            }
            Stage::Type => {
                let ty = match params.ty {
                    Some(ref ty) => Some(ty.to_string()),
                    None => with_raw(|| match prompt::TypePrompt::new(&config).run() {
                        prompt::TypePromptResult::Type(ty) => Some(ty),
                        prompt::TypePromptResult::Terminate => std::process::exit(2),
                        prompt::TypePromptResult::Escape => None,
                    }),
                };

                let ty = match ty {
                    Some(s) => s,
                    None => {
                        stage = Stage::Files;
                        continue;
                    }
                };

                stage = Stage::Scope(ty);
            }
            Stage::Scope(ty) => {
                let scope = match params.scope {
                    Some(ref scope) => Some((Some(scope.to_string()), 0)),
                    None => with_raw(|| match prompt::ScopePrompt::new(&config, &ty).run() {
                        prompt::ScopePromptResult::Scope(scope, lines) => Some((scope, lines)),
                        prompt::ScopePromptResult::Terminate => std::process::exit(2),
                        prompt::ScopePromptResult::Escape => None,
                    }),
                };

                let (scope, lines) = match scope {
                    Some(t) => t,
                    None => {
                        stage = Stage::Type;
                        continue;
                    }
                };

                stage = Stage::Message(ty, scope);
                escape_clear_lines = lines as u16;
            }
            Stage::Message(ty, scope) => {
                let message = match params.message {
                    Some(ref message) => Some(message.to_string()),
                    None => with_raw(|| match prompt::MessagePrompt::new(&config).run() {
                        prompt::MessagePromptResult::Message(message) => Some(message),
                        prompt::MessagePromptResult::Terminate => std::process::exit(2),
                        prompt::MessagePromptResult::Escape => None,
                    }),
                };

                let message = match message {
                    Some(s) => s,
                    None => {
                        stage = Stage::Scope(ty);

                        let mut stderr = std::io::stderr();
                        ct::queue!(
                            stderr,
                            cursor::MoveUp(escape_clear_lines + 2),
                            terminal::Clear(terminal::ClearType::FromCursorDown)
                        )
                        .unwrap();

                        continue;
                    }
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
