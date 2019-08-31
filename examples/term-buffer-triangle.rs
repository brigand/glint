use clint::TermBuffer;

enum Direction {
    Grow,
    Shrink,
}

const MIN: usize = 1;
const MAX: usize = 30;

struct State(usize, Direction);

impl State {
    fn update(&mut self) -> usize {
        match self.1 {
            Direction::Grow => {
                if self.0 >= MAX {
                    self.0 -= 1;
                    self.1 = Direction::Shrink
                } else {
                    self.0 += 1;
                }
            }
            Direction::Shrink => {
                if self.0 <= MIN {
                    self.0 = 1;
                    self.1 = Direction::Grow
                } else {
                    self.0 -= 1;
                }
            }
        }
        self.0
    }
}

pub fn main() {
    let mut state = State(0, Direction::Grow);
    let mut buf = TermBuffer::default();

    loop {
        let count = state.update();
        // let count = MAX;

        for i in 0..count {
            let line = format!("|{}\\", "*".repeat(i));
            buf.push_line(line);
        }
        buf.push_line(format!(" {}", "‾".repeat(count - 1)));

        buf.render_frame();
        buf.flush();
        std::thread::sleep_ms(15);
    }
}
