// parse the template
use std::str::FromStr;
use std::vec::Vec;

#[derive(Debug)]
pub enum LineAction {
    Next,
    Continue,
}

#[derive(Debug)]
pub enum RecordAction {
    NoRecord,
    Record,
    Clear,
    ClearAll,
}

#[derive(Debug)]
pub enum Value {
    Value,
    Flags(Option<Vec<ValueOption>>),
    Name(String),
    Regex(String),
}

#[derive(Debug)]
pub enum ValueOption {
    Filldown,
    Key,
    Required,
    List,
    Fillup,
}

#[derive(Debug)]
pub struct TemplateValue<'a> {
    name: &'a str,
    options: Vec<ValueOption>,
    regex: &'a str,
}

impl FromStr for ValueOption {
    type Err = ();

    fn from_str(i: &str) -> Result<ValueOption, ()> {
        match i {
            "Filldown" => Ok(ValueOption::Filldown),
            "Key" => Ok(ValueOption::Key),
            "Required" => Ok(ValueOption::Required),
            "List" => Ok(ValueOption::List),
            "Fillup" => Ok(ValueOption::Fillup),
            _ => Err(()),
        }
    }
}

pub mod parser {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_till1, take_until},
        character::complete::{multispace0, multispace1},
        combinator::{all_consuming, map},
        multi::{many_till, separated_list0},
        sequence::tuple,
        IResult,
    };

    // TODO: Doesn't technically follow the spec
    pub fn parse_value_section(i: &str) -> IResult<&str, Vec<TemplateValue>> {
        let (remaining, (values, _)) = many_till(
            //alt((parse_value_line, alt((consume_comment, tag("\n"))))),
            map(
                tuple((
                    alt((take_until("Value"), consume_comment)),
                    parse_value_line,
                )),
                |(_, v)| v,
            ),
            tag("\n"),
        )(i)?;
        Ok((remaining, values))
    }

    fn consume_comment(i: &str) -> IResult<&str, &str> {
        let (remaining, (_, _, _, _)) =
            tuple((multispace0, tag("#"), take_till1(|c| c == '\n'), tag("\n")))(i)?;
        Ok((remaining, remaining))
    }

    fn parse_value_line(i: &str) -> IResult<&str, TemplateValue> {
        let (remaining, (_, _, flags, _, name, _, regex, _)) = tuple((
            tag("Value"),
            multispace1,
            parse_optional_values,
            multispace0,
            take_till1(|c| c == ' '),
            multispace1,
            take_till1(|c| c == '\n'),
            tag("\n"),
        ))(i)?;
        Ok((
            remaining,
            TemplateValue {
                name: name.trim(),
                options: flags,
                regex: regex.trim(),
            },
        ))
    }

    fn parse_optional_values(i: &str) -> IResult<&str, Vec<ValueOption>> {
        separated_list0(tag(","), optional_flag)(i)
    }

    fn optional_flag(i: &str) -> IResult<&str, ValueOption> {
        let (i, flag) = alt((
            tag("Filldown"),
            tag("Key"),
            tag("Required"),
            tag("List"),
            tag("Fillup"),
        ))(i)?;
        Ok((i, ValueOption::from_str(flag).unwrap()))
    }

    pub fn parse_template(i: &str) -> IResult<&str, Vec<TemplateValue>> {
        match all_consuming(parse_value_section)(i) {
            Ok((_, v)) => Ok((i, v)),
            Err(e) => Err(e),
        }
    }
}
