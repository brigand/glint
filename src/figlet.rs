use crossterm as ct;
use std::fmt::Display;

const CHAR_OFFSET: usize = 32;

#[derive(Debug, Clone)]
pub struct Font {
    height: usize,
    chars: Vec<Char>,
}

#[derive(Debug, Clone)]
pub struct Char {
    width: usize,
    text: Vec<String>,
}

impl Font {
    /// Creates a Vec for holding figlet output with enough vertical space
    /// to contain this font.
    /// You may borrow this as a mutable slice and pass it to `Font::write_to_buf`
    pub fn create_vec(&self) -> Vec<String> {
        (0..self.height).map(|_| String::new()).collect()
    }

    /// Writes a single character to the buffer.
    /// The length of `output` should be at least `font.height()`, however it's safe to
    /// pass a smaller slice (the rendering will be cropped).
    /// The same number of characters will be appeneded to each string, padding with spaces if needed.
    ///
    /// #Example
    /// ```norun
    /// let mut output = font.create_vec();
    /// for c in "feat(client)".chars() {
    ///     font.write_to_buf(c, &mut output[..])
    ///         .expect("write_to_buf should return the width");
    /// }
    /// ```
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

    pub fn write_to_buf_color<'a, R: Display>(
        &'a self,
        c: char,
        output: &mut [String],
        style: impl Fn(&'a str) -> R,
    ) -> Option<usize> {
        let c = self.chars.get(c as usize - CHAR_OFFSET)?;

        for (src, dest) in c.text.iter().zip(output.iter_mut()) {
            let c = &src[0..c.width];
            let c = format!(
                "{}{}",
                style(c),
                ct::style("").with(ct::Color::Reset).on(ct::Color::Reset)
            );

            dest.push_str(&c);
        }

        Some(c.width)
    }

    /// Returns the height of the largest character in the font.
    /// This operation is very fast.
    pub fn height(&self) -> usize {
        self.height
    }
}

/// Takes an iterator over lines of a .flf (figlet) file, and attempts to parse
/// it into a Font, which can be used for rendering.
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
