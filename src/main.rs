mod cli;

mod color;
mod config;
mod prompt;

pub use config::Config;

fn main() {
    let args = cli::parse();
    let mut config = Config::default();

    let ty = match args.value_of("TYPE") {
        Some(ty) => ty.to_string(),
        None => match prompt::TypePrompt::new(&mut config).run() {
            prompt::TypePromptResult::Type(ty) => ty,
            _ => panic!("no result received"),
        },
    };

    println!("ty: {}", ty);
}
