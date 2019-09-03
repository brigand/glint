use clap::{clap_app, AppSettings};

pub struct Commit {
    pub message: Option<String>,
    pub ty: Option<String>,
    pub scope: Option<String>,
    pub git_args: Vec<String>,
}

pub enum Cli {
    Commit(Commit),
}

fn get_app() -> clap::App<'static, 'static> {
    clap_app!(clint =>
        (version: "0.1")
        (author: "Frankie Bagnardi <f.bagnardi@gmail.com>")
        (about: "A friendly commitlint CLI. You probably want the 'commit' subcommand, or 'c' for short.")
        (@subcommand commit =>
        (@arg message: -m --message +takes_value "Specifies the commit message (optional; otherwise interactive prompt)")
        (@arg type: -t --type +takes_value "Sets the 'type' component of the commitlint (optional; otherwise interactive prompt)")
        (@arg scope: -s --scope +takes_value "Sets the 'scope' component of the commitlint (optional; otherwise interactive prompt)")
        (@arg all: -a --all "Allows passing --all to git without the -- separator.")
        (@arg GIT_ARGS: [GIT_ARGS]... "Arguments which will be passed to 'git commit'. Pass a '--' argument before the git args to disable special parsing.")
        )

    )
    // .setting(AppSettings::SubcommandsNegateReqs)
    .setting(AppSettings::TrailingVarArg)
    .setting(AppSettings::InferSubcommands)
    .setting(AppSettings::SubcommandRequiredElseHelp)
}

fn get_git_args(args: &clap::ArgMatches) -> Vec<String> {
    let mut git_args = args.values_of_lossy("GIT_ARGS").unwrap_or_default();

    if args.is_present("all") {
        git_args.insert(0, "-a".into());
    }

    git_args
}

pub fn parse() -> Cli {
    let app = get_app();
    let args = app.get_matches();

    match args.subcommand() {
        ("commit", Some(args)) => Cli::Commit(Commit {
            message: args.value_of("message").map(String::from),
            ty: args.value_of("type").map(String::from),
            scope: args.value_of("scope").map(String::from),
            git_args: get_git_args(args),
        }),
        _ => {
            eprintln!("{}", args.usage());
            std::process::exit(1);
        }
    }
}
