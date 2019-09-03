use clap::{clap_app, AppSettings};

static DEFAULT_COMMAND: &str = "commit";

pub struct Commit {
        pub message: Option<String>,
        pub ty: Option<String>,
        pub scope: Option<String>,
        pub git_args: Vec<String>,
    }

pub enum Cli {
    Commit(Commit)
}

fn get_app() -> clap::App<'static, 'static> {
 clap_app!(clint =>
        (version: "0.1")
        (author: "Frankie Bagnardi <f.bagnardi@gmail.com>")
        (about: "A friendly commitlint CLI")
        (@subcommand commit =>
        (@arg message: -m --message +takes_value "Specifies the commit message (optional; otherwise interactive prompt)")
        (@arg type: -t --type +takes_value "Sets the 'type' component of the commitlint (optional; otherwise interactive prompt)")
        (@arg scope: -s --scope +takes_value "Sets the 'scope' component of the commitlint (optional; otherwise interactive prompt)")
        (@arg GIT_ARGS: [GIT_ARGS]...)
        )

    )
    // .setting(AppSettings::SubcommandsNegateReqs)
    .setting(AppSettings::TrailingVarArg)
    .setting(AppSettings::InferSubcommands)


}

pub fn parse() -> Cli {
    let app = get_app();
    let args = app.get_matches();

match args.subcommand() {
    ("commit", Some(args)) => Cli::Commit(Commit {
        message: args.value_of("message").map(String::from),
        ty: args.value_of("type").map(String::from),
        scope: args.value_of("scope").map(String::from),
        git_args: args.values_of_lossy("GIT_ARGS").unwrap_or_default(),
    }),
    _ => {
        eprintln!("{}", args.usage());
        std::process::exit(1);
    }
}
}
