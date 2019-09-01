use clint::figlet;
use std::fs::read_to_string;

pub fn main() {
    let figlet_src = read_to_string("src/big.flf").expect("src/big.flf must exist");
    let font = figlet::parse(figlet_src.lines()).expect("should be able to parse font");

    let mut output = font.create_vec();

    // println!("{:#?}", font);

    let mut x_offset = 0;

    for c in "feat(client)".chars() {
        let width = font
            .write_to_buf(c, &mut output[..], x_offset)
            .expect("write_to_buf should return the width");

        x_offset += width;
    }

    for line in output {
        println!("{}", line);
    }
}
