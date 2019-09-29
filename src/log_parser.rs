use memchr::memchr;

pub enum ParserElement {
    Word,
    BracketDelimited,
    QuoteDelimited,
}
use ParserElement::*;

impl ParserElement {
    fn parse_word<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match memchr(b' ', &input[..].as_bytes()) {
            Some(next_pos) => {
                let next_pos = next_pos;
                return Ok((&input[..next_pos], next_pos));
            },
            None => {
                return Err(UnexpectedEndOfLine);
            }
        }
    }

    fn parse_bracket_delimited<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        if input.as_bytes()[0] != b'[' {
            return Err(UnexpectedCharacter('[', input.as_bytes()[0] as char, 0));
        }

        match memchr(b']', &input[..].as_bytes()) {
            Some(next_pos) => {
                return Ok((&input[1..next_pos], next_pos + 1))
            },
            None => {
                return Err(UnexpectedEndOfLine);
            }
        }
    }

    fn parse_quote_delimited<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        if input.as_bytes()[0] != b'"' {
            return Err(UnexpectedCharacter('"', input.as_bytes()[0] as char, 0));
        }

        let mut cur_pos = 1;
        while let Some(next_pos) = memchr(b'"', &input[cur_pos..].as_bytes()) {
            let next_pos = next_pos + cur_pos;
            if input.as_bytes()[next_pos - 1] != b'\\' {
                return Ok((&input[1..next_pos], next_pos + 1));
            }
            cur_pos = next_pos + 1;
        }
        return Err(UnexpectedEndOfLine);
    }

    fn parse<'a>(&self, input: &'a str) -> Result<(&'a str, usize), ParserError> {
        match self {
            Word => self.parse_word(input),
            BracketDelimited => self.parse_bracket_delimited(input),
            QuoteDelimited => self.parse_quote_delimited(input),
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
    element_type: ParserElement
}

impl LogField {
    pub fn log_format_combined() -> Vec<LogField> {
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
                name: String::from("status"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("responsesize"),
                element_type: ParserElement::Word,
            },
            LogField {
                name: String::from("referer"),
                element_type: ParserElement::QuoteDelimited,
            },
            LogField {
                name: String::from("useragent"),
                element_type: ParserElement::QuoteDelimited,
            },
        ]
    }
}

pub struct LineParser<'a> {
    log_format: &'a Vec<LogField>,
    field_ids: Option<Vec<usize>>,
    max_field_id: usize
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
            max_field_id
        }
    }

    fn get_parse_result(&self, input: &'b str) -> Result<Vec<&'b str>, ParserError> {
        let mut pos: usize = 0;
        let mut result = Vec::with_capacity(self.max_field_id + 1);

        for (id, field) in self.log_format.iter().enumerate() {
            if id > self.max_field_id {
                break;
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
            },
            None => Ok(result),
        }
    }
}
