use clint::figlet;
use crossterm as ct;
use std::fs::read_to_string;

pub fn main() {
    let figlet_src = read_to_string("src/big.flf").expect("src/big.flf must exist");
    let font = figlet::parse(figlet_src.lines()).expect("should be able to parse font");

    let mut output = font.create_vec();

    // println!("{:#?}", font);

    for c in "feat".chars() {
        font.write_to_buf_color(c, &mut output[..], |s| ct::style(s).with(ct::Color::Red))
            .expect("write_to_buf should return the width");
    }

    for c in "(".chars() {
        font.write_to_buf(c, &mut output[..])
            .expect("write_to_buf should return the width");
    }

    for c in "client".chars() {
        font.write_to_buf_color(c, &mut output[..], |s| ct::style(s).with(ct::Color::Blue))
            .expect("write_to_buf should return the width");
    }

    for c in ")".chars() {
        font.write_to_buf(c, &mut output[..])
            .expect("write_to_buf should return the width");
    }

    for line in output {
        println!("{}", line);
    }
}
