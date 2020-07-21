use crossterm::{
    self as ct,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
};

fn with_raw<R>(f: impl FnOnce() -> R) -> R {
    match ct::terminal::enable_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(_raw_screen) => {
            let r = f();
            let _ignored = ct::terminal::disable_raw_mode();
            r
        }
    }
}

fn main() {
    with_raw(|| loop {
        if let Ok(ev) = event::read() {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers,
            }) = ev
            {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    println!("Exiting");
                    break;
                }
            }

            println!("{:?}", ev);
            println!("{}", ct::cursor::MoveLeft(200));
        }
    })
}
