mod log_parser;

use std::io::Write;

use std::io::BufRead;
use std::io::BufWriter;

use structopt::StructOpt;

/// This program reads Apache log data from standard input, processes it, and
/// writes it to standard output.
#[derive(StructOpt, Debug)]
struct Opt {
    /// The character to insert between each output field. [default: tab]
    #[structopt(value_name="CHAR", short="d", long="delimiter")]
    field_delimiter: Option<char>,

    /// A whitespace delimited list of fields to extract from each line.
    #[structopt(value_name="FIELD", short, long)]
    fields: Option<Vec<String>>,

    /// Whether or not to color fields using ANSI color codes.
    #[structopt(short, long, possible_values=&["always","auto","never"], default_value="auto")]
    color: String,
}

fn write_output<T: Write>(writer: &mut T, result: Vec<&str>, delimiter: char) -> std::io::Result<()> {
    writer.write(result[0].as_bytes())?;
    for i in 1..result.len() {
        writer.write(&[delimiter as u8])?;
        writer.write(result[i].as_bytes())?;
    }
    writer.write(&[b'\n'])?;
    Ok(())
}

fn write_output_color<T: Write>(writer: &mut T, result: Vec<&str>, delimiter: char) -> std::io::Result<()> {
    let colors = vec![
        "\u{001b}[31m", // red
        "\u{001b}[32m", // green
        "\u{001b}[33m", // yellow
        "\u{001b}[34m", // blue
        "\u{001b}[35m", // magenta
        "\u{001b}[36m", // cyan
    ];
    let reset = "\u{001b}[0m";

    writer.write(colors[0].as_bytes())?;
    writer.write(result[0].as_bytes())?;
    writer.write(reset.as_bytes())?;
    for i in 1..result.len() {
        writer.write(&[delimiter as u8])?;
        writer.write(colors[i % colors.len()].as_bytes())?;
        writer.write(result[i].as_bytes())?;
        writer.write(reset.as_bytes())?;
    }
    writer.write(&[b'\n'])?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    let opt = Opt::from_args();

    let format = log_parser::LogField::log_format_combined();
    let parser = log_parser::LineParser::new(&format, opt.fields);
    let delimiter = opt.field_delimiter.unwrap_or('\t');

    let use_color = match &opt.color[..] {
        "auto" => atty::is(atty::Stream::Stdout),
        "always" => true,
        "never" => false,
        _ => unreachable!(),
    };

    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap();
        if let Ok(v) = parser.parse_line(&line) {
            if use_color {
                write_output_color(&mut stdout, v, delimiter)?;
            } else {
                write_output(&mut stdout, v, delimiter)?;
            }
        }
    }

    Ok(())
}
