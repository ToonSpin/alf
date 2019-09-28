use memchr::memchr;

pub enum ParserElement {
    Word,
    BracketDelimited,
    QuoteDelimited,
}
use ParserElement::*;

pub enum ParserError {
    UnexpectedCharacter(char, char, usize),
    UnexpectedEndOfLine,
}
use ParserError::*;

pub struct LineParser<'a> {
    format: &'a Vec<ParserElement>
}

impl<'a, 'b> LineParser<'a> {
    pub fn new(format: &'a Vec<ParserElement>) -> LineParser {
        LineParser {
            format
        }
    }

    pub fn parse_line(&self, input: &'b str) -> Result<Vec<&'b str>, ParserError> {
        let mut pos: usize = 0;
        let mut result = Vec::with_capacity(self.format.len());
        for element in self.format.iter() {
            match element {
                Word => {
                    match memchr(b' ', &input[pos..].as_bytes()) {
                        Some(next_pos) => {
                            let next_pos = next_pos + pos;
                            result.push(&input[pos..next_pos]);
                            pos = next_pos + 1;
                        },
                        None => {
                            return Err(UnexpectedEndOfLine);
                        }
                    }
                },
                BracketDelimited => {
                    if input.as_bytes()[pos] != b'[' {
                        return Err(UnexpectedCharacter('[', input.as_bytes()[pos] as char, pos));
                    }
                    pos += 1;
                    match memchr(b']', &input[pos..].as_bytes()) {
                        Some(next_pos) => {
                            let next_pos = next_pos + pos;
                            result.push(&input[pos..next_pos]);
                            pos = next_pos + 2;
                        },
                        None => {
                            return Err(UnexpectedEndOfLine);
                        }
                    }
                },
                QuoteDelimited => {
                    if input.as_bytes()[pos] != b'"' {
                        return Err(UnexpectedCharacter('"', input.as_bytes()[pos] as char, pos));
                    }
                    pos += 1;
                    match memchr(b'"', &input[pos..].as_bytes()) {
                        Some(next_pos) => {
                            let next_pos = next_pos + pos;
                            result.push(&input[pos..next_pos]);
                            pos = next_pos + 2;
                        },
                        None => {
                            return Err(UnexpectedEndOfLine);
                        }
                    }
                },
            }
        }
        Ok(result)
    }
}
