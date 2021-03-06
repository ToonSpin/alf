use crate::log_parser::LineParser;
use std::io::{BufRead, BufWriter, Write};
use structopt::StructOpt;

mod log_parser;

/// Alf, short for "Apache Log Format", reads Apache log data from standard
/// input, processes it, and writes it to standard output.
#[derive(StructOpt)]
struct Opt {
    /// The character to insert between each output field [default: tab]
    #[structopt(value_name = "char", short = "d", long = "delimiter")]
    field_delimiter: Option<char>,

    /// A whitespace delimited list of fields to extract from each line
    #[structopt(value_name = "field", short, long)]
    fields: Option<Vec<String>>,

    /// The log format this program should expect
    #[structopt(short = "F", long, possible_values = &["combined","combinedio","common","vhost_common",], default_value = "combined")]
    format: String,

    /// Whether or not to color fields using ANSI color codes
    #[structopt(short, long, possible_values = &["always","auto","never"], default_value = "auto")]
    color: String,

    /// Lists the available fields instead of parsing input
    #[structopt(short = "l", long = "list-fields")]
    list_fields: bool,
}

// allowing the clippy::unused_io_amount lint, because using write_all rather
// than write leads to a performance hit.
#[allow(clippy::unused_io_amount)]
fn write_line<T: Write>(writer: &mut T, result: Vec<&str>, sep: char) -> std::io::Result<()> {
    writer.write(result[0].as_bytes())?;
    for field in result.iter().skip(1) {
        writer.write(&[sep as u8])?;
        writer.write(field.as_bytes())?;
    }
    writer.write(&[b'\n'])?;
    Ok(())
}

// allowing the clippy::unused_io_amount lint, because using write_all rather
// than write leads to a performance hit.
#[allow(clippy::unused_io_amount)]
fn write_line_color<T: Write>(writer: &mut T, result: Vec<&str>, sep: char) -> std::io::Result<()> {
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
        writer.write(&[sep as u8])?;

        writer.write(colors[i % colors.len()].as_bytes())?;
        writer.write(result[i].as_bytes())?;
        writer.write(reset.as_bytes())?;
    }

    writer.write(&[b'\n'])?;
    Ok(())
}

#[inline]
fn parse_and_print(parser: &LineParser, delimiter: char, use_color: bool) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());

    for (line_number, line) in std::io::stdin().lock().lines().enumerate() {
        let line = line.unwrap();
        match parser.parse_line(&line) {
            Ok(v) => {
                if !v.is_empty() {
                    if use_color {
                        write_line_color(&mut stdout, v, delimiter)?;
                    } else {
                        write_line(&mut stdout, v, delimiter)?;
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{} on line {}",
                    LineParser::get_error_string(e),
                    line_number
                );
            }
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let format = match &opt.format[..] {
        "combined" => log_parser::LogField::log_format_combined(),
        "combinedio" => log_parser::LogField::log_format_combinedio(),
        "common" => log_parser::LogField::log_format_common(),
        "vhost_common" => log_parser::LogField::log_format_vhost_common(),
        _ => unreachable!(),
    };

    if opt.list_fields {
        for name in log_parser::LogField::get_names(&format) {
            println!("{}", name);
        }
        Ok(())
    } else {
        let parser = log_parser::LineParser::new(&format, opt.fields);
        let delimiter = opt.field_delimiter.unwrap_or('\t');

        let use_color = match &opt.color[..] {
            "always" => true,
            "auto" => atty::is(atty::Stream::Stdout),
            "never" => false,
            _ => unreachable!(),
        };

        parse_and_print(&parser, delimiter, use_color)
    }
}
