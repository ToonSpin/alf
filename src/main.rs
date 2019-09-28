use std::io::Write;

use std::io::BufRead;
use std::io::BufWriter;

mod log_parser;

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    let format_combined = vec![
        log_parser::ParserElement::Word,
        log_parser::ParserElement::Word,
        log_parser::ParserElement::Word,
        log_parser::ParserElement::BracketDelimited,
        log_parser::ParserElement::QuoteDelimited,
        log_parser::ParserElement::Word,
        log_parser::ParserElement::Word,
        log_parser::ParserElement::QuoteDelimited,
        log_parser::ParserElement::QuoteDelimited,
    ];

    let parser = log_parser::LineParser::new(&format_combined);

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if let Ok(v) = parser.parse_line(&line) {
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
