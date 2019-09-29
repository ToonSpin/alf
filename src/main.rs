mod log_parser;

use std::io::Write;

use std::io::BufRead;
use std::io::BufWriter;

use structopt::StructOpt;

/// This program reads Apache log data from standard input, processes it, and
/// writes it to standard output.
#[derive(StructOpt, Debug)]
struct Opt {
    /// A character to insert between each output field. [default: tab]
    #[structopt(value_name="CHAR", short="d", long="delimiter")]
    field_delimiter: Option<char>,

    /// A whitespace delimited list of fields to extract from each line.
    #[structopt(value_name="FIELD", short, long)]
    fields: Option<Vec<String>>,
}


fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    let opt = Opt::from_args();

    let format = log_parser::LogField::log_format_combined();
    let parser = log_parser::LineParser::new(&format, opt.fields);
    let delimiter = opt.field_delimiter.unwrap_or('\t') as u8;

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if let Ok(v) = parser.parse_line(&line) {
            stdout.write(v[0].as_bytes())?;
            for i in 1..v.len() {
                stdout.write(&[delimiter])?;
                stdout.write(v[i].as_bytes())?;
            }
            stdout.write(&[b'\n'])?;
        }
    }

    Ok(())
}
