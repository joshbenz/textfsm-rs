// parse the template
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;
use std::vec::Vec;

static MATCH_ACTION_STR: &'static str = r"(?P<match>.*)(\s->(?P<action>.*))";
static LINE_OP_STR: &'static str = r"(?P<ln_op>Continue|Next|Error)";
static RECORD_STR: &'static str = r"(?P<rec_op>Clear|Clearall|Record|NoRecord)";
static NEWSTATE_STR: &'static str = r#"(?P<new_state>\w+|\".*\")"#;

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
        values: &HashMap<String, TemplateValue>,
    ) -> Result<TemplateRule, ()> {
        lazy_static! {
            static ref LINE_OP_OPT_STR: String= format!(r"({}(\.{})?)", LINE_OP_STR, RECORD_STR);
            static ref LINE_AND_RECORD_ACTION_STR: String= format!(r"^\s+{}(\s+{})?$", LINE_OP_STR, RECORD_STR);
            static ref LINE_RECORD_OPT_STR: String = format!(r"^\s+{}(\s+{})?$", RECORD_STR, NEWSTATE_STR);
            static ref DEFAULT_OP_OPT_NEWSTATE_STR: String = format!(r"^(\s+{})?$", NEWSTATE_STR);

            // Implicit default is '(regexp) -> Next.NoRecord'
            static ref MATCH_ACTION: Regex = Regex::new(MATCH_ACTION_STR).unwrap();

            // Line operators.
            static ref LINE_OP_RE: Regex = Regex::new(LINE_OP_STR).unwrap();

            // Record operators.
            static ref RECORD_RE: Regex = Regex::new(RECORD_STR).unwrap();

            // Line operator with optional record operator.
            static ref LINE_OP_OPT_RE: Regex = Regex::new(&LINE_OP_OPT_STR).unwrap();

            // New State or 'Error' string.
            static ref NEWSTATE_RE: Regex = Regex::new(NEWSTATE_STR).unwrap();

            // Compound operator (line and record) with optional new state.
            static ref ACTION_RE: Regex = Regex::new(&LINE_AND_RECORD_ACTION_STR).unwrap();

            // Record operator with optional new state.
            static ref ACTION2_RE: Regex = Regex::new(&LINE_RECORD_OPT_STR).unwrap();

            // Default operators with optional new state.
            static ref ACTION3_RE: Regex = Regex::new(&DEFAULT_OP_OPT_NEWSTATE_STR).unwrap();
        }

        let rule_line = rule_line.trim();
        if rule_line.len() == 0 {
            // TODO err no rule
        }

        Err(())
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
    let mut map: HashMap<String, Vec<Result<TemplateRule, ()>>> = HashMap::default();

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
    Err(())
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
