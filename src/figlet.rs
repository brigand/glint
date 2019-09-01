const CHAR_OFFSET: usize = 32;

#[derive(Debug, Clone)]
pub struct Font {
    height: usize,
    pub chars: Vec<Char>,
}

#[derive(Debug, Clone)]
pub struct Char {
    width: usize,
    pub text: Vec<String>,
}

impl Font {
    pub fn create_vec(&self) -> Vec<String> {
        (0..self.height).map(|_| String::new()).collect()
    }

    pub fn write_to_buf(&self, c: char, output: &mut [String]) -> Option<usize> {
        let c = self.chars.get(c as usize - CHAR_OFFSET)?;

        for (src, dest) in c.text.iter().zip(output.iter_mut()) {
            for i in 0..c.width {
                let c = src.as_bytes().get(i).map(|&c| c).unwrap_or(b' ');

                dest.push(c as char);
            }
        }

        Some(c.width)
    }
}

pub fn parse<'a>(mut iter: impl Iterator<Item = &'a str>) -> Option<Font> {
    let header: Vec<_> = iter.next()?.split(" ").collect();

    let height: usize = header.get(1)?.parse().ok()?;
    let hard_blank = header.get(0)?.chars().last()?;
    let comments: usize = header.get(5)?.parse().ok()?;

    let mut iter = iter.skip(comments);

    let mut chars: Vec<Char> = Vec::new();
    let mut current: Vec<String> = Vec::with_capacity(height);

    loop {
        match iter.next() {
            Some(line) => {
                let mut len = line.len();
                for c in line.as_bytes().iter().rev() {
                    if *c == b'@' {
                        len -= 1;
                    } else {
                        break;
                    }
                }

                if len == line.len() {
                    // NOTE: we only care about the positional characters for now
                    // so stop when we reach e.g. "160  NO-BREAK SPACE"
                    break;
                }
                let slice = &line[0..len];
                current.push(slice.replace(hard_blank, " "));

                if current.len() == height {
                    let width = current.iter().fold(0, |max, s| std::cmp::max(max, s.len()));

                    if current.len() == height {
                        chars.push(Char {
                            text: std::mem::replace(&mut current, Vec::with_capacity(height)),
                            width,
                        })
                    }
                }
            }
            None => break,
        };
    }

    Some(Font { height, chars })
}
