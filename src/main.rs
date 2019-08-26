mod cli;

mod color;
mod config;
mod prompt;
mod term_buffer;

pub use config::Config;
pub use term_buffer::TermBuffer;

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
        Some(ty) => ty.to_string(),
        None => with_raw(|_raw| match prompt::TypePrompt::new(&mut config).run() {
            prompt::TypePromptResult::Type(ty) => ty,
            prompt::TypePromptResult::Terminate => {
                eprintln!("\nExiting.");
                std::process::exit(1);
            }
            _ => panic!("no result received"),
        }),
    };

    println!("ty: {}", ty);
}
