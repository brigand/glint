use std::env::current_dir;
use std::ffi::OsStr;
use std::fmt;
use std::io::Cursor;
use std::io::Read;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread::spawn;

mod parse_log;

pub use parse_log::LogItem;

#[derive(Debug, Clone)]
pub struct Git {
    cwd: PathBuf,
    repo_root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct GitStatus(pub Vec<GitStatusItem>);

#[derive(Debug, Clone)]
pub struct GitStatusItem {
    file_name: String,
    staged: Option<GitStatusType>,
    unstaged: Option<GitStatusType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GitStatusType {
    Added,
    Modified,
    Renamed,
    Untracked,
    Deleted,
    None,
}

#[derive(Debug)]
pub enum GitError {
    NotGitRepo,
    Io(io::Error),
}

impl Git {
    pub fn from_cwd() -> Result<Self, GitError> {
        let cwd = current_dir().map_err(GitError::Io)?;

        let mut repo_root = None;

        for dir in cwd.ancestors() {
            if dir.join(".git").is_dir() {
                repo_root = Some(dir.into());

                break;
            }
        }

        match repo_root {
            Some(repo_root) => Ok(Git { cwd, repo_root }),
            None => Err(GitError::NotGitRepo),
        }
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

    pub fn log<I>(&self, other_args: impl IntoIterator<Item = I>) -> Command
    where
        I: AsRef<OsStr>,
    {
        let mut command = Command::new("git");

        // Setup
        command.current_dir(&self.cwd);
        command.stdin(Stdio::null());

        // Args
        command.arg("log");
        for arg in other_args {
            command.arg(arg);
        }
        command.arg("--raw");
        command.arg("--pretty=raw");

        command
    }

    pub fn log_parsed<I>(&self, other_args: impl IntoIterator<Item = I>) -> io::Result<Vec<LogItem>>
    where
        I: AsRef<OsStr>,
    {
        let proc = self.log(other_args).stdout(Stdio::piped()).spawn()?;
        let stdout = proc.stdout.expect("must be able to access stdout");
        Ok(parse_log::parse_logs(
            BufReader::new(stdout).lines().filter_map(Result::ok),
        ))
    }

    /// Stages files using `git add`. Run from the repo root.gs
    pub fn add<I>(&self, files: impl IntoIterator<Item = I>) -> Command
    where
        I: AsRef<OsStr>,
    {
        let mut command = Command::new("git");

        // Setup
        command.current_dir(&self.repo_root);
        command.stdin(Stdio::null());

        // Args
        command.arg("add");
        command.arg("--");

        for file in files {
            command.arg(file.as_ref());
        }

        command
    }

    pub fn less(&self, file: impl AsRef<OsStr>) -> io::Result<()> {
        Command::new("less")
            .arg(file.as_ref())
            .current_dir(&self.repo_root)
            .status()?;

        Ok(())
    }

    /// Prints the
    pub fn directory_untracked_less(&self, dir: &Path) -> io::Result<()> {
        let ls = Command::new("git")
            .current_dir(&self.repo_root)
            .arg("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .arg("--")
            .arg(dir)
            .stdout(Stdio::piped())
            .spawn()?;

        let ls_stdout = ls.stdout.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "failed to get stdout of git diff")
        })?;

        let message = format!(
            "= Contents of {} =",
            String::from_utf8_lossy(dir.as_bytes())
        );
        let prefix = format!(
            "{bar}\n{message}\n{bar}\n\n",
            message = message,
            bar = "=".repeat(message.len())
        );

        let mut less = Command::new("less")
            .arg("-R")
            .current_dir(&self.repo_root)
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = less.stdin.take().expect("Failed to open stdin");
        spawn(move || {
            let suffix = "\n= End =\n";
            let prefix = Cursor::new(prefix);

            let mut input = prefix.chain(ls_stdout).chain(Cursor::new(suffix));

            let _r = io::copy(&mut input, &mut stdin);
        });

        less.wait()?;

        Ok(())
    }

    pub fn diff_less<I>(&self, files: impl IntoIterator<Item = I>) -> io::Result<()>
    where
        I: AsRef<OsStr>,
    {
        let diff = Command::new("git")
            .current_dir(&self.repo_root)
            .arg("diff")
            .arg("--color=always")
            .arg("--")
            .args(files.into_iter())
            .stdout(Stdio::piped())
            .spawn()?;

        Command::new("less")
            .arg("-R")
            .current_dir(&self.repo_root)
            .stdin(diff.stdout.ok_or_else(|| {
                io::Error::new(io::ErrorKind::Other, "failed to get stdout of git diff")
            })?)
            .status()?;

        Ok(())
    }

    pub fn status(&self) -> io::Result<GitStatus> {
        let mut command = Command::new("git");

        // Setup
        command.current_dir(&self.cwd);
        command.stdout(Stdio::piped());

        // Args
        command.arg("status");
        command.arg("--porcelain");

        let stdout = command.spawn()?.stdout.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Could not capture standard output.")
        })?;

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
                        file_name: file,
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
    pub fn new(file_name: String) -> Self {
        GitStatusItem {
            file_name,
            staged: None,
            unstaged: None,
        }
    }
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
    pub fn status(&self) -> &GitStatusType {
        self.unstaged.as_ref().unwrap_or(&GitStatusType::None)
    }

    pub fn is_dir(&self) -> bool {
        self.file_name.ends_with('/')
    }

    pub fn is_new(&self) -> bool {
        self.staged.is_none() && matches!(self.unstaged, Some(GitStatusType::Untracked))
    }
}

impl Into<String> for GitStatusItem {
    fn into(self) -> String {
        (&self).into()
    }
}

impl Into<String> for &'_ GitStatusItem {
    fn into(self) -> String {
        self.file_name().into()
    }
}

impl GitStatusType {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            'A' => Some(GitStatusType::Added),
            'M' => Some(GitStatusType::Modified),
            'R' => Some(GitStatusType::Renamed),
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
