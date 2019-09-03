use std::env::current_dir;
use std::ffi::OsStr;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Git {
  cwd: PathBuf,
}

pub enum GitError {
  NotGitRepo,
  Io(io::Error),
}

impl Git {
  pub fn from_cwd() -> Result<Self, GitError> {
    let cwd = current_dir().map_err(GitError::Io)?;

    let mut is_git_repo = false;

    for dir in cwd.ancestors() {
      if dir.join(".git").is_dir() {
        is_git_repo = true;
        break;
      }
    }

    if !is_git_repo {
      return Err(GitError::NotGitRepo);
    }

    Ok(Git { cwd })
  }

  pub fn commit<I>(&self, message: &str, other_args: impl IntoIterator<Item = I>) -> Command
  where
    I: AsRef<OsStr>,
  {
    let mut command = Command::new("git");

    // Setup
    command.current_dir(&self.cwd);
    command.stdin(Stdio::null());

    // Args
    command.arg("commit");
    command.arg("-m");
    command.arg(message);
    for arg in other_args {
      command.arg(arg);
    }

    command
  }
}

impl fmt::Display for GitError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      GitError::NotGitRepo => write!(f, "This directory is not a git repository."),
      GitError::Io(err) => write!(f, "Internal I/O error: {}", err),
    }
  }
}
