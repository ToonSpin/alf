use std::io::Write;
use memchr::memchr;

use std::io::BufRead;
use std::io::BufWriter;

enum ParserElement {
    Word,
    BracketDelimited,
    QuoteDelimited,
}

enum ParserError {
    UnexpectedCharacter(char, char, usize),
    UnexpectedEndOfLine,
}

fn parse<'a>(format: &Vec<ParserElement>, input: &'a str) -> Result<Vec<&'a str>, ParserError> {
    let mut pos: usize = 0;
    let mut result = Vec::with_capacity(format.len());
    for element in format.iter() {
        match element {
            ParserElement::Word => {
                match memchr(b' ', &input[pos..].as_bytes()) {
                    Some(next_pos) => {
                        let next_pos = next_pos + pos;
                        result.push(&input[pos..next_pos]);
                        pos = next_pos + 1;
                    },
                    None => {
                        return Err(ParserError::UnexpectedEndOfLine);
                    }
                }
            },
            ParserElement::BracketDelimited => {
                if input.as_bytes()[pos] != b'[' {
                    return Err(ParserError::UnexpectedCharacter('[', input.as_bytes()[pos] as char, pos));
                }
                pos += 1;
                match memchr(b']', &input[pos..].as_bytes()) {
                    Some(next_pos) => {
                        let next_pos = next_pos + pos;
                        result.push(&input[pos..next_pos]);
                        pos = next_pos + 2;
                    },
                    None => {
                        return Err(ParserError::UnexpectedEndOfLine);
                    }
                }
            },
            ParserElement::QuoteDelimited => {
                if input.as_bytes()[pos] != b'"' {
                    return Err(ParserError::UnexpectedCharacter('"', input.as_bytes()[pos] as char, pos));
                }
                pos += 1;
                match memchr(b'"', &input[pos..].as_bytes()) {
                    Some(next_pos) => {
                        let next_pos = next_pos + pos;
                        result.push(&input[pos..next_pos]);
                        pos = next_pos + 2;
                    },
                    None => {
                        return Err(ParserError::UnexpectedEndOfLine);
                    }
                }
            },
        }
    }
    Ok(result)
}

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    let format_combined = vec![
        ParserElement::Word,
        ParserElement::Word,
        ParserElement::Word,
        ParserElement::BracketDelimited,
        ParserElement::QuoteDelimited,
        ParserElement::Word,
        ParserElement::Word,
        ParserElement::QuoteDelimited,
        ParserElement::QuoteDelimited,
    ];

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if let Ok(v) = parse(&format_combined, &line) {
            stdout.write(v[0].as_bytes())?;
            for i in 1..v.len() {
                stdout.write(&[b'\t'])?;
                stdout.write(v[i].as_bytes())?;
            }
            stdout.write(&[b'\n'])?;
        }
    }

    Ok(())
}
