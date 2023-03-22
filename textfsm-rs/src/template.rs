// parse the template
use std::collections::HashMap;
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
pub enum ValueOption {
    Filldown,
    Key,
    Required,
    List,
    Fillup,
}

#[derive(Debug)]
pub struct TemplateValue {
    pub name: String,
    pub options: Option<Vec<ValueOption>>,
    pub regex: String,
}

pub struct TemplateState<'a> {
    pub name: &'a str,
    pub rules: Vec<&'a str>,
}

pub struct TemplateRule {
    pub regex: String,
    pub line_op: LineAction,
    pub record_op: RecordAction,
    pub new_state: String,
}

impl TemplateRule {
    fn from_template_line(
        rule_line: &str,
        values: &HashMap<String, TemplateState>,
    ) -> TemplateRule {
        Self {
            regex: "",
            line_op: (),
            record_op: (),
            new_state: (),
        }
    }
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

pub fn parse_template(template: &str) -> Result<(), ()> {
    let mut lines = template.lines();
    let template_values = parse_value_section(&mut lines)?;
    let k = parse_state_section(&mut lines, &template_values);
    Ok(())
}

fn parse_state_section<'a, I>(
    lines: &mut I,
    values: &HashMap<String, TemplateValue>,
) -> Result<HashMap<String, TemplateRule>, ()>
where
    I: Iterator<Item = &'a str>,
{
    let mut map = HashMap::default();

    while let Some(l) = lines.next() {
        let l_no_spaces = l.trim();
        if !l_no_spaces.starts_with("#") {
            // template should have states followed by rules
            let state = l;
            let mut rules = std::vec::Vec::default();
            while let Some(rule_line) = lines.next() {
                //TODO sub value regex
                // syntax allows for 1 space, 2 spaces or tabs for indentation
                if rule_line.starts_with(" ")
                    || rule_line.starts_with("  ")
                    || rule_line.starts_with("\t")
                {
                    let rule_line = rule_line.trim();
                    rules.push(TemplateRule::from_template_line(rule_line, values));
                }
            }
            map.insert(state.to_string(), rules);

            /*
            // syntax allows for 1 space, 2 spaces or tabs for indentation
            if l.starts_with(" ") || l.starts_with("  ") || l.starts_with("\t") {
                // this is a rule
            } else if let Some(c) = l.chars().next() {
                if c.is_ascii_alphanumeric() {
                    let state = l;
                } else {
                    // TODO: error state doesnt start with ascii alphanumeric
                }
            } else {
                // TODO
                // some sort of syntax error
            }*/
        }
    }
    Ok(map)
}

fn parse_value_section<'a, I>(lines: &mut I) -> Result<HashMap<String, TemplateValue>, ()>
where
    I: Iterator<Item = &'a str>,
{
    let mut map = HashMap::default();

    while let Some(l) = lines.next() {
        let l = l.trim();
        if !l.starts_with("#") {
            if l.len() == 0 {
                break;
            }
            let parts: Vec<&str> = l.split(" ").collect();

            // TODO syntax Error
            if parts[0] != "Value " {
                return Err(());
            }

            // We have optonal fields
            if parts.len() > 3 {
                map.insert(
                    parts[2].to_string(),
                    TemplateValue {
                        name: parts[2].trim().to_owned(),
                        options: Some(
                            parts[1]
                                .split(",")
                                .map(ValueOption::from_str)
                                .collect::<Result<Vec<_>, _>>()?,
                        ),
                        regex: parts[3].trim().to_owned(),
                    },
                );
            } else {
                map.insert(
                    parts[1].to_string(),
                    TemplateValue {
                        name: parts[1].trim().to_owned(),
                        options: None,
                        regex: parts[2].trim().to_owned(),
                    },
                );
            }
        }
    }
    Ok(map)
}
