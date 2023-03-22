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
    pub name: &'a str,
    pub options: Option<Vec<ValueOption>>,
    pub regex: &'a str,
}

pub struct TemplateState<'a> {
    pub name: &'a str,
    pub rules: Vec<&'a str>,
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

pub fn parse_template(template: &str) {
    let mut lines = template.lines();
    let template_values = parse_value_section(&mut lines);
}

fn parse_value_section<'a, I>(lines: &mut I) -> Result<Vec<TemplateValue<'a>>, ()>
where
    I: Iterator<Item = &'a str>,
{
    let mut res = std::vec::Vec::default();

    while let Some(l) = lines.next() {
        let l = l.trim();
        if !l.starts_with("#") {
            if l.len() == 0 {
                break;
            }
            let parts: Vec<&str> = l.split(" ").collect();

            // We have optonal fields
            if parts.len() > 3 {
                res.push(TemplateValue {
                    name: parts[2].trim(),
                    options: Some(
                        parts[1]
                            .split(",")
                            .map(ValueOption::from_str)
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    regex: parts[3].trim(),
                });
            } else {
                res.push(TemplateValue {
                    name: parts[1].trim(),
                    options: None,
                    regex: parts[2].trim(),
                });
            }
        }
    }
    Ok(res)
}
