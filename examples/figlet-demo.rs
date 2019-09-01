use clint::figlet;
use std::fs::read_to_string;

pub fn main() {
    let figlet_src = read_to_string("src/big.flf").expect("src/big.flf must exist");
    let font = figlet::parse(figlet_src.lines()).expect("should be able to parse font");

    let mut output = font.create_vec();

    // println!("{:#?}", font);

    for c in "feat(client)".chars() {
        font.write_to_buf(c, &mut output[..])
            .expect("write_to_buf should return the width");
    }

    for line in output {
        println!("{}", line);
    }
}
