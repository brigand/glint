use crossterm::{self as ct, queue};
use std::io::{self, Write as _Write};

fn print_cursor() {
    let (x, y) = ct::cursor().pos();
    println!("({}, {})", x, y);
}

fn print_lines(count: usize) {
    for i in 0..count {
        println!("Example line {}/{}", i + 1, count);
    }
}

pub fn main() {
    let mut stdout = io::stdout();
    print_cursor();
    print_lines(5);
    print_cursor();

    std::thread::sleep_ms(2000);

    let (x, y) = ct::cursor().pos();
    queue!(
        stdout,
        ct::Goto(x, y - 3),
        ct::Clear(ct::ClearType::FromCursorDown)
    );
    stdout.flush();

    std::thread::sleep_ms(1000);

    print_cursor();
}
