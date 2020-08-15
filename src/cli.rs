use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Commit {
    /// Sets the 'type' component of the commit (optional; otherwise interactive prompt)
    #[structopt(short, long)]
    pub ty: Option<String>,

    /// Sets the 'scope' component of the commit (optional; otherwise interactive prompt)
    #[structopt(short, long)]
    pub scope: Option<String>,

    /// Sets the main message component of the commit (optional; otherwise interactive prompt)
    #[structopt(short, long)]
    pub message: Option<String>,

    #[structopt(short, long)]
    pub all: bool,

    /// Arguments which will be passed to 'git commit'.
    /// Pass a '--' argument before the git args to disable special parsing.
    #[structopt(short, long)]
    pub git_args: Vec<String>,
}

#[derive(StructOpt)]
pub struct Log {
    /// Filter by 'type' e.g. 'feat'
    #[structopt(short, long)]
    pub ty: Option<String>,

    /// Filter by 'scope' e.g. 'client'
    #[structopt(short, long)]
    pub scope: Option<String>,

    /// Number of commits to display.
    #[structopt(short, long)]
    pub num: Option<usize>,

    /// Output log info as a JSON array of objects
    #[structopt(short, long)]
    pub json: bool,

    /// Only useful when filing bug reports for glint.
    #[structopt(short, long)]
    pub debug: bool,

    /// Arguments which will be passed to 'git log'. Note that certain
    /// options will break the command. Passing file paths/prefixes is
    /// typical usage.
    pub git_args: Vec<String>,
}

/// A friendly conventional commit tool. You probably want the 'commit' subcommand, or 'c' for short.
#[derive(StructOpt)]
pub enum Cli {
    /// Create a new commit
    Commit(Commit),

    /// View recent commits
    Log(Log),
}

pub fn parse() -> Cli {
    let matches = Cli::clap()
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::InferSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    Cli::from_clap(&matches)
}
