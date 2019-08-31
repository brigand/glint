mod cli;

use clint::{prompt, Config};

fn with_raw<R>(f: impl FnOnce(crossterm::RawScreen) -> R) -> R {
    match crossterm::RawScreen::into_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(raw_screen) => f(raw_screen),
    }
}

fn main() {
    let args = cli::parse();
    let mut config = Config::default();

    let ty = match args.value_of("TYPE") {
        Some(ty) => Some(ty.to_string()),
        None => with_raw(|_raw| match prompt::TypePrompt::new(&mut config).run() {
            prompt::TypePromptResult::Type(ty) => Some(ty),
            prompt::TypePromptResult::Terminate => None,
            _ => panic!("no result received"),
        }),
    };

    let ty = match ty {
        Some(ty) => ty,
        None => std::process::exit(1),
    };

    println!("ty: {}", ty);
}
