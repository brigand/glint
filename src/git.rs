use std::env::current_dir;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub struct Git {
  cwd: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GitStatus(pub Vec<GitStatusItem>);

#[derive(Debug, Clone)]
pub struct GitStatusItem {
  file: String,
  staged: Option<GitStatusType>,
  unstaged: Option<GitStatusType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GitStatusType {
  Added,
  Modified,
  Untracked,
  Deleted,
}

#[derive(Debug)]
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

  pub fn status(&self) -> io::Result<GitStatus> {
    let mut command = Command::new("git");

    // Setup
    command.current_dir(&self.cwd);
    command.stdout(Stdio::piped());

    // Args
    command.arg("status");
    command.arg("--porcelain");

    let stdout = command
      .spawn()?
      .stdout
      .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Could not capture standard output."))?;

    let items = BufReader::new(stdout)
      .lines()
      .filter_map(|line| line.ok())
      .filter_map(|line| {
        let mut chars = line.chars();
        let staged = chars
          .next()
          .and_then(GitStatusType::from_char)
          .filter(|item| match item {
            GitStatusType::Untracked => false,
            _ => true,
          });
        let unstaged = chars.next().and_then(GitStatusType::from_char);

        chars.next();
        let file: String = chars.collect();

        if file.is_empty() {
          None
        } else {
          Some(GitStatusItem {
            file,
            staged,
            unstaged,
          })
        }
      })
      .collect();

    Ok(GitStatus(items))
  }
}
impl GitStatus {
  pub fn iter(&self) -> impl Iterator<Item = &GitStatusItem> {
    self.0.iter()
  }

  pub fn any_staged(&self) -> bool {
    self.iter().any(|item| item.staged.is_some())
  }

  pub fn any_unstaged(&self) -> bool {
    self.iter().any(|item| item.unstaged.is_some())
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }
}

impl GitStatusItem {
  pub fn file(&self) -> &str {
    &self.file
  }
}

impl Into<String> for GitStatusItem {
  fn into(self) -> String {
    (&self).into()
  }
}

impl Into<String> for &'_ GitStatusItem {
  fn into(self) -> String {
    self.file().into()
  }
}

impl GitStatusType {
  pub fn from_char(ch: char) -> Option<Self> {
    match ch {
      'A' => Some(GitStatusType::Added),
      'M' => Some(GitStatusType::Modified),
      'D' => Some(GitStatusType::Deleted),
      '?' => Some(GitStatusType::Untracked),
      _ => None,
    }
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
