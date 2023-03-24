// parse the template
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::str::FromStr;
use std::vec::Vec;

#[derive(Debug)]
pub enum LineAction {
    Next,
    Continue,
    Empty,
}

impl FromStr for LineAction {
    type Err = ();

    fn from_str(i: &str) -> Result<LineAction, ()> {
        match i {
            "Next" => Ok(LineAction::Next),
            "Continue" => Ok(LineAction::Continue),
            "" => Ok(LineAction::Empty),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum RecordAction {
    NoRecord,
    Record,
    Clear,
    ClearAll,
    Empty,
}

impl FromStr for RecordAction {
    type Err = ();

    fn from_str(i: &str) -> Result<RecordAction, ()> {
        match i {
            "Record" => Ok(RecordAction::Record),
            "NoRecord" => Ok(RecordAction::NoRecord),
            "Clear" => Ok(RecordAction::Clear),
            "CLearAll" => Ok(RecordAction::ClearAll),
            "" => Ok(RecordAction::Empty),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub enum ValueOption {
    Filldown,
    Key,
    Required,
    List,
    Fillup,
    Invalid,
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
            _ => Ok(ValueOption::Invalid),
        }
    }
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
            // Implicit default is '(regexp) -> Next.NoRecord'
            static ref MATCH_ACTION: Regex = Regex::new(r"(?P<match>.*)(\s->(?P<action>.*))").unwrap();

            // Line operators.
            static ref OPER_RE: Regex = Regex::new(r"(?P<ln_op>Continue|Next|Error)").unwrap();

            // Record operators.
            static ref RECORD_RE: Regex = Regex::new(r"(?P<rec_op>Clear|Clearall|Record|NoRecord)").unwrap();

            // Line operator with optional record operator.
            static ref OPER_RECORD_RE: Regex = Regex::new(&format!(r"({}(\.{})?)", OPER_RE.as_str(), RECORD_RE.as_str())).unwrap();

            // New State or 'Error' string.
            static ref NEWSTATE_RE: Regex = Regex::new(r#"(?P<new_state>\w+|\".*\")"#).unwrap();

            // Compound operator (line and record) with optional new state.
            static ref ACTION_RE: Regex = Regex::new(&format!(r#""^\s+{}(\s+{})?$""#, OPER_RECORD_RE.as_str(), NEWSTATE_RE.as_str())).unwrap();

            // Record operator with optional new state.
            static ref ACTION2_RE: Regex = Regex::new(&format!(r#"^\s+{}(\s+{})?$"#, RECORD_RE.as_str(), NEWSTATE_RE.as_str())).unwrap();

            // Default operators with optional new state.
            static ref ACTION3_RE: Regex = Regex::new(&format!(r#"^(\s+{})?$"#, NEWSTATE_RE.as_str())).unwrap();

            // Used for Error
            static ref ERROR_RE: Regex = Regex::new(r#"\w+"#).unwrap();
        }

        let mut the_match = "";
        let mut regex = "";
        let mut line_op = "";
        let mut record_op = "";
        let mut new_state = "";

        let rule_line = rule_line.trim();
        if rule_line.len() == 0 {
            // TODO err no rule
        }

        let match_action = MATCH_ACTION.captures(rule_line);
        match match_action {
            Some(ref m) => {
                match m.name("match") {
                    Some(group) => the_match = group.as_str(),
                    None => {} //TODO Err?
                }
            }
            None => the_match = rule_line,
        };

        regex = the_match;
        let match_action = match_action.unwrap();

        // TODO String interpolate ${VAR} from values

        let action_re = match match_action.name("action") {
            Some(action) => ACTION_RE
                .captures(action.as_str())
                .or(ACTION2_RE.captures(action.as_str()))
                .or(ACTION3_RE.captures(action.as_str())),
            None => None, //TODO Err, no action?
        };

        if let Some(action) = action_re {
            if let Some(m) = action.name("ln_op") {
                line_op = m.as_str();
            }
            if let Some(m) = action.name("rec_op") {
                record_op = m.as_str();
            }
            if let Some(m) = action.name("new_state") {
                new_state = m.as_str();
            }
        }

        if line_op == "Continue" && new_state.len() > 0 {
            // TDOD ERROR
        }

        if line_op != "Error" && new_state.len() > 0 {
            if ERROR_RE.is_match(new_state) {
                // TODO ERROR
            }
        }

        Ok(TemplateRule {
            regex: regex.to_string(),
            line_op: LineAction::from_str(line_op).unwrap(),
            record_op: RecordAction::from_str(record_op).unwrap(),
            new_state: new_state.to_string(),
        })
    }
}

//fn interpolate(template: &str, values: HashMap<String, TemplateValue>) -> String {}

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
