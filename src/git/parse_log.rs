#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LogItem {
    commit: String,
    epoch_secs: i64,
    message: String,
    files: Vec<String>,
}

enum Parser {
    SeekingHeader,
    Header {
        commit: String,
    },
    Header2 {
        commit: String,
        epoch_secs: i64,
    },
    PreMessage {
        commit: String,
        epoch_secs: i64,
    },
    Message {
        commit: String,
        epoch_secs: i64,
        message: String,
    },

    Footer {
        commit: String,
        epoch_secs: i64,
        message: String,
        files: Vec<String>,
    },
    Complete(Option<LogItem>),
    Void,
}

impl Parser {
    fn take(&mut self) -> Self {
        std::mem::replace(self, Parser::Void)
    }

    fn handle_line(&mut self, line: String) {
        use Parser::*;

        let liner = line.as_str();
        let is_blank = line.is_empty();
        let leading_space = line.chars().next().filter(|c| c.is_whitespace()).is_some();

        let state = self.take();
        *self = match state {
            SeekingHeader if line.starts_with("commit ") => Header {
                commit: line["commit ".len()..].to_string(),
            },
            Header { .. } if liner.starts_with("committer ") => {
                let mut time_word = None;
                for word in liner.split_whitespace().rev() {
                    if !word.is_empty()
                        && word.chars().next().filter(|c| c.is_ascii_digit()).is_some()
                    {
                        if let Ok(epoch_secs) = word.parse() {
                            time_word = Some(epoch_secs);
                            break;
                        }
                    }
                }

                if let Header { commit } = state {
                    match time_word {
                        Some(epoch_secs) => Header2 { commit, epoch_secs },
                        _ => Header { commit },
                    }
                } else {
                    unreachable!()
                }
            }
            Header2 { commit, epoch_secs } => {
                if is_blank {
                    PreMessage { commit, epoch_secs }
                } else {
                    Header2 { commit, epoch_secs }
                }
            }
            PreMessage { commit, epoch_secs } => {
                let message = if line.chars().take(4).all(|c| c.is_whitespace()) {
                    let (i, _) = line.char_indices().skip(4).next().unwrap();
                    liner[i..].to_string()
                } else {
                    line
                };

                Message {
                    commit,
                    epoch_secs,
                    message,
                }
            }
            Message { .. } if is_blank => {
                if let Message {
                    commit,
                    epoch_secs,
                    message,
                } = state
                {
                    Footer {
                        commit,
                        epoch_secs,
                        message,
                        files: vec![],
                    }
                } else {
                    unreachable!()
                }
            }
            Message {
                commit,
                epoch_secs,
                mut message,
            } => {
                let addition = if line.chars().take(4).all(|c| c.is_whitespace()) {
                    let (i, _) = line.char_indices().skip(4).next().unwrap();
                    &liner[i..]
                } else {
                    liner
                };

                message.push('\n');
                message.push_str(addition);

                Message {
                    commit,
                    epoch_secs,
                    message,
                }
            }

            Footer {
                commit,
                epoch_secs,
                message,
                mut files,
            } => {
                if line.starts_with(":") {
                    if let Some(name) = liner.split_whitespace().last() {
                        files.push(name.to_string());
                    }
                }

                if is_blank {
                    Complete(Some(LogItem {
                        commit,
                        epoch_secs,
                        message,
                        files,
                    }))
                } else {
                    Footer {
                        commit,
                        epoch_secs,
                        message,
                        files,
                    }
                }
            }
            x => x,
        };
    }
}

pub fn parse_logs<'a>(mut lines: impl Iterator<Item = String>) -> Vec<LogItem> {
    let mut parser = Parser::SeekingHeader;
    let mut items = vec![];

    for line in lines {
        parser.handle_line(line);

        parser = match parser {
            Parser::Complete(None) => Parser::SeekingHeader,
            Parser::Complete(Some(item)) => {
                items.push(item);
                Parser::SeekingHeader
            }
            _ => parser,
        };
    }

    items
}

#[cfg(test)]
mod test {
    use super::{parse_logs, LogItem};
    use std::io::{BufRead, BufReader};

    // Note: the whitespace here is important, and there is
    // one trailing blank line. The sanity check enforces this
    static RAW: &str = r#"commit 18d90e52cf8d6a486bee299b3949ebd213c85f2a
tree f221c23e63d1fe5b52d5acf39599fa02e2a69fc0
parent 089918cea42077b499ff092113ced60451214912
author Frankie Bagnardi <f.bagnardi@gmail.com> 1568585467 -0700
committer Frankie Bagnardi <f.bagnardi@gmail.com> 1568585467 -0700

    docs(gif): updates usage gif

:100644 100644 6bbe237 4fe5fc6 M        assets/usage.gif

"#;

    #[test]
    fn sanity() {
        assert_eq!(&RAW[0..6], "commit");
        assert_eq!(&RAW[RAW.len() - 2..], "\n\n");
    }

    #[test]
    fn parse_initial() {
        let lines = BufReader::new(RAW.as_bytes())
            .lines()
            .filter_map(|r| r.ok());
        let logs = parse_logs(lines);
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0],
            LogItem {
                commit: "18d90e52cf8d6a486bee299b3949ebd213c85f2a".into(),
                epoch_secs: 1568585467,
                message: "docs(gif): updates usage gif".into(),
                files: vec!["assets/usage.gif".into()],
            }
        );
    }
}
