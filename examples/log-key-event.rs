use crossterm as ct;

fn with_raw<R>(f: impl FnOnce(crossterm::RawScreen) -> R) -> R {
    match ct::RawScreen::into_raw_mode() {
        Err(_) => {
            eprintln!("Failed to convert stdio to raw mode. Can't continue.");
            std::process::exit(1);
        }
        Ok(raw_screen) => f(raw_screen),
    }
}

fn main() {
    with_raw(|_raw| {
        let input = crossterm::input();
        let mut sync_stdin = input.read_sync();

        loop {
            let event = sync_stdin.next();
            if let Some(ev) = event {
                if let ct::InputEvent::Keyboard(ct::KeyEvent::Ctrl('c')) = ev {
                    println!("Exiting");
                    break;
                }
                println!("{:?}", ev);
                ct::cursor().move_left(100);
            }
        }
    })
}
