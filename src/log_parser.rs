use memchr::memchr;

pub enum ParserElement {
    Word,
    BracketDelimited,
    QuoteDelimited,
    Computed,
}
use ParserElement::*;

impl ParserElement {
    fn parse_word<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match memchr(b' ', &input[..].as_bytes()) {
            Some(next_pos) => {
                let next_pos = next_pos;
                Ok((&input[..next_pos], next_pos))
            }
            None => Ok((&input[..], input.len())),
        }
    }

    fn parse_bracket_delimited<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        if input.as_bytes()[0] != b'[' {
            return Err(UnexpectedCharacter('[', input.as_bytes()[0] as char, 0));
        }

        match memchr(b']', &input[..].as_bytes()) {
            Some(next_pos) => Ok((&input[1..next_pos], next_pos + 1)),
            None => Err(UnexpectedEndOfLine),
        }
    }

    fn get_end_quote_pos(&self, input: &str) -> Result<usize, ParserError> {
        let mut cur_pos = 0;
        while let Some(next_pos) = memchr(b'"', &input[cur_pos..].as_bytes()) {
            let next_pos = cur_pos + next_pos;
            if next_pos == 0 || input.as_bytes()[next_pos - 1] != b'\\' {
                return Ok(next_pos);
            }
            cur_pos = next_pos + 1;
        }
        Err(UnexpectedEndOfLine)
    }

    fn parse_quote_delimited<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        if input.as_bytes()[0] != b'"' {
            return Err(UnexpectedCharacter('"', input.as_bytes()[0] as char, 0));
        }
        let end_pos = self.get_end_quote_pos(&input[1..])? + 1;
        Ok((&input[1..end_pos], end_pos + 1))
    }

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match self {
            Word => self.parse_word(input),
            BracketDelimited => self.parse_bracket_delimited(input),
            QuoteDelimited => self.parse_quote_delimited(input),
            Computed => unreachable!(),
        }
    }
}

pub enum ParserError {
    UnexpectedCharacter(char, char, usize),
    UnexpectedEndOfLine,
}
use ParserError::*;

pub struct LogField {
    name: String,
    element_type: ParserElement,
}

impl LogField {
    pub fn get_names(format: &[LogField]) -> Vec<String> {
        format.iter().map(|f| f.name.clone()).collect()
    }

    pub fn log_format_common() -> Vec<LogField> {
        vec![
            LogField {
                name: String::from("ip"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("rfc1413"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("username"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("time"),
                element_type: ParserElement::BracketDelimited,
            },
            LogField {
                name: String::from("request"),
                element_type: ParserElement::QuoteDelimited,
            },
            LogField {
                name: String::from("method"),
                element_type: ParserElement::Computed,
            },
            LogField {
                name: String::from("uri"),
                element_type: ParserElement::Computed,
            },
            LogField {
                name: String::from("http"),
                element_type: ParserElement::Computed,
            },
            LogField {
                name: String::from("status"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("responsesize"),
                element_type: ParserElement::Word,
            },
        ]
    }

    pub fn log_format_vhost_common() -> Vec<LogField> {
        let mut log_format = vec![LogField {
            name: String::from("vhost"),
            element_type: ParserElement::Word,
        }];
        log_format.append(&mut Self::log_format_common());
        log_format
    }

    pub fn log_format_combined() -> Vec<LogField> {
        let mut log_format = Self::log_format_common();
        log_format.append(&mut vec![
            LogField {
                name: String::from("referer"),
                element_type: ParserElement::QuoteDelimited,
            },
            LogField {
                name: String::from("useragent"),
                element_type: ParserElement::QuoteDelimited,
            },
        ]);
        log_format
    }

    pub fn log_format_combinedio() -> Vec<LogField> {
        let mut log_format = Self::log_format_combined();
        log_format.append(&mut vec![
            LogField {
                name: String::from("input"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("output"),
                element_type: ParserElement::Word,
            },
        ]);
        log_format
    }
}

pub struct LineParser<'a> {
    log_format: &'a [LogField],
    field_ids: Option<Vec<usize>>,
    max_field_id: usize,
}

impl<'a, 'b> LineParser<'a> {
    pub fn new(log_format: &[LogField], fields: Option<Vec<String>>) -> LineParser {
        let mut field_ids = None;
        let mut max_field_id = log_format.len() - 1;
        if let Some(v) = fields {
            max_field_id = 0;
            let mut ids = Vec::new();
            for field_name in v {
                for (i, field) in log_format.iter().enumerate() {
                    if field_name == field.name {
                        ids.push(i);
                        if i > max_field_id {
                            max_field_id = i;
                        }
                        break;
                    }
                }
            }
            field_ids = Some(ids);
        }
        LineParser {
            log_format,
            field_ids,
            max_field_id,
        }
    }

    fn get_computed_fields(field_match: &str, pos: usize) -> Result<(usize, usize), ParserError> {
        let first_space;
        let second_space;
        if let Some(p) = memchr(b' ', field_match.as_bytes()) {
            first_space = p;
            if let Some(p) = memchr(b' ', field_match[first_space + 1..].as_bytes()) {
                second_space = first_space + 1 + p;
            } else {
                return Err(UnexpectedCharacter(' ', '\"', pos + field_match.len()));
            }
        } else {
            return Err(UnexpectedCharacter(' ', '\"', pos + field_match.len()));
        }

        Ok((first_space, second_space))
    }

    fn get_parse_result(&self, input: &'b str) -> Result<Vec<&'b str>, ParserError> {
        let mut pos: usize = 0;
        let mut result = Vec::with_capacity(self.max_field_id + 1);

        for (id, field) in self.log_format.iter().enumerate() {
            if id > self.max_field_id {
                break;
            }

            if pos >= input.as_bytes().len() {
                return Err(UnexpectedEndOfLine);
            }

            if let ParserElement::Computed = field.element_type {
                continue;
            }

            let (field_match, consumed) = field.element_type.parse(&input[pos..])?;
            if field.name == "request" {
                if self.field_ids.is_some() {
                    result.push(field_match);
                }
                if field_match == "-" {
                    result.push(&field_match[0..1]);
                    result.push(&field_match[0..1]);
                    result.push(&field_match[0..1]);
                } else {
                    let (first_space, second_space) = Self::get_computed_fields(field_match, pos)?;
                    result.push(&field_match[..first_space]);
                    result.push(&field_match[first_space + 1..second_space]);
                    result.push(&field_match[second_space + 1..]);
                }
            } else {
                result.push(field_match);
            }
            pos += consumed;

            if input.as_bytes().len() > pos && input.as_bytes()[pos] != b' ' {
                return Err(UnexpectedCharacter(' ', input.as_bytes()[pos] as char, pos));
            }
            pos += 1;
        }
        Ok(result)
    }

    pub fn parse_line(&self, input: &'b str) -> Result<Vec<&'b str>, ParserError> {
        let result = self.get_parse_result(input)?;
        match &self.field_ids {
            Some(v) => {
                let mut selected_result = Vec::new();
                for i in v.iter() {
                    selected_result.push(result[*i]);
                }
                Ok(selected_result)
            }
            None => Ok(result),
        }
    }

    pub fn get_error_string(e: ParserError) -> String {
        match e {
            UnexpectedEndOfLine => String::from("Unexpected end of line"),
            UnexpectedCharacter(expected, got, _position) => format!(
                "Unexpected character: expected '{}', got '{}'",
                expected, got
            ),
        }
    }
}
