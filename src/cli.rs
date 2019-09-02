use clap::clap_app;

pub fn parse() -> clap::ArgMatches<'static> {
    clap_app!(clint =>
        (version: "0.1")
        (author: "Frankie Bagnardi <f.bagnardi@gmail.com>")
        (about: "A friendly commitlint CLI")
        (@subcommand commit =>
        (@arg MESSAGE: -m --message +takes_value "Specifies the commit message (optional; otherwise interactive prompt)")
        (@arg TYPE: -t --type +takes_value "Sets the 'type' component of the commitlint (optional; otherwise interactive prompt)")
        (@arg SCOPE: -s --scope +takes_value "Sets the 'scope' component of the commitlint (optional; otherwise interactive prompt)")
        )

    ).get_matches()
}
