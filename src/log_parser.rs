use memchr::memchr;

pub enum ParserElement {
    Word,
    BracketDelimited,
    QuoteDelimited(bool, bool),
}
use ParserElement::*;

impl ParserElement {
    fn parse_word<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match memchr(b' ', &input[..].as_bytes()) {
            Some(next_pos) => {
                let next_pos = next_pos;
                return Ok((&input[..next_pos], next_pos));
            }
            None => {
                return Ok((&input[..], input.len()));
            }
        }
    }

    fn parse_bracket_delimited<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        if input.as_bytes()[0] != b'[' {
            return Err(UnexpectedCharacter('[', input.as_bytes()[0] as char, 0));
        }

        match memchr(b']', &input[..].as_bytes()) {
            Some(next_pos) => return Ok((&input[1..next_pos], next_pos + 1)),
            None => {
                return Err(UnexpectedEndOfLine);
            }
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
        return Err(UnexpectedEndOfLine);
    }

    fn parse_quote_delimited<'a>(
        &self,
        input: &'a str,
        left: bool,
        right: bool,
    ) -> Result<(&'a str, usize), ParserError> {
        let mut cur_pos = 0;
        if left {
            if input.as_bytes()[0] != b'"' {
                return Err(UnexpectedCharacter('"', input.as_bytes()[0] as char, 0));
            }
            cur_pos += 1;
        }

        let end_pos = self.get_end_quote_pos(&input[cur_pos..])? + cur_pos;

        if right {
            return Ok((&input[cur_pos..end_pos], end_pos + 1));
        } else {
            if let Some(next_space) = memchr(b' ', &input[cur_pos..].as_bytes()) {
                let next_space = next_space + cur_pos;
                if next_space < end_pos {
                    return Ok((&input[cur_pos..next_space], next_space));
                } else {
                    return Err(UnexpectedCharacter(' ', '\"', end_pos));
                }
            }
        }
        return Err(UnexpectedEndOfLine);
    }

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match self {
            Word => self.parse_word(input),
            BracketDelimited => self.parse_bracket_delimited(input),
            QuoteDelimited(left, right) => self.parse_quote_delimited(input, *left, *right),
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
                name: String::from("method"),
                element_type: ParserElement::QuoteDelimited(true, false),
            },
            LogField {
                name: String::from("request"),
                element_type: ParserElement::QuoteDelimited(false, false),
            },
            LogField {
                name: String::from("http"),
                element_type: ParserElement::QuoteDelimited(false, true),
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
                element_type: ParserElement::QuoteDelimited(true, true),
            },
            LogField {
                name: String::from("useragent"),
                element_type: ParserElement::QuoteDelimited(true, true),
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
    log_format: &'a Vec<LogField>,
    field_ids: Option<Vec<usize>>,
    max_field_id: usize,
}

impl<'a, 'b> LineParser<'a> {
    pub fn new(log_format: &'a Vec<LogField>, fields: Option<Vec<String>>) -> LineParser {
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

            let (field_match, consumed) = field.element_type.parse(&input[pos..])?;
            result.push(field_match);
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
            UnexpectedCharacter(expected, got, _position) => String::from(format!(
                "Unexpected character: expected '{}', got '{}'",
                expected, got
            )),
        }
    }
}
