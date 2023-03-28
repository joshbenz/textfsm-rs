pub mod fsm;
pub mod template;
use std::collections::HashMap;

use either::{Either, Left, Right};
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Error in the FSM state execution")]
    TextFsmError,
    #[error("Errors while parsing templates")]
    TextFSMTemplateError,
}

pub enum StateChangeTrigger {
    FsmAction,
    SkipRecord,
    SkipValue,
    Continue,
}

pub trait TextFsmOption {
    fn on_create_options(&mut self);
    fn on_clear_var(&mut self);
    fn on_clear_all_var(&mut self);
    fn on_assign_var(&mut self);
    fn on_get_value(&mut self);
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()>;
}

pub struct TextFsmValue {
    compiled_regex: Regex,
    max_name_len: u32,
    name: String,
    options: Vec<String>,
    regex: String,
    template: String,
    fsm: String,
    value: Option<String>,
    values: Vec<Either<HashMap<String, String>, String>>,
}

pub struct Required(TextFsmValue);
impl TextFsmOption for Required {
    fn on_create_options(&mut self) {}
    fn on_clear_var(&mut self) {}
    fn on_clear_all_var(&mut self) {}
    fn on_assign_var(&mut self) {}
    fn on_get_value(&mut self) {}
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()> {
        match self.0.value {
            Some(_) => Err(()),
            None => Ok(StateChangeTrigger::SkipRecord),
        }
    }
}

pub struct Filldown(TextFsmValue, Option<String>);
impl TextFsmOption for Filldown {
    fn on_create_options(&mut self) {
        self.1 = None;
    }
    fn on_clear_var(&mut self) {
        match self.1 {
            Some(v) => self.0.value = Some(v),
            None => {}
        }
    }
    fn on_clear_all_var(&mut self) {
        self.1 = None
    }
    fn on_assign_var(&mut self) {
        match self.0.value {
            Some(v) => self.1 = Some(v),
            None => {}
        }
    }
    fn on_get_value(&mut self) {}
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()> {
        Err(())
    }
}

pub struct Fillup(TextFsmValue);
impl TextFsmOption for Fillup {
    fn on_create_options(&mut self) {}
    fn on_clear_var(&mut self) {}
    fn on_clear_all_var(&mut self) {}
    fn on_assign_var(&mut self) {
        match self.0.value {
            Some(v) => {
                /*
                if self.value.value:
                  # Get index of relevant result column.
                  value_idx = self.value.fsm.values.index(self.value)
                  # Go up the list from the end until we see a filled value.
                  # pylint: disable=protected-access
                  for result in reversed(self.value.fsm._result):
                    if result[value_idx]:
                      # Stop when a record has this column already.
                      break
                    # Otherwise set the column value.
                    result[value_idx] = self.value.value
                           *
                           * */
            }
            None => {}
        }
    }
    fn on_get_value(&mut self) {}
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()> {
        Err(())
    }
}
pub struct Key(TextFsmValue);
impl TextFsmOption for Key {
    fn on_create_options(&mut self) {}
    fn on_clear_var(&mut self) {}
    fn on_clear_all_var(&mut self) {}
    fn on_assign_var(&mut self) {}
    fn on_get_value(&mut self) {}
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()> {
        Err(())
    }
}
pub struct List(Vec<Either<HashMap<String, String>, String>>, TextFsmValue);
impl TextFsmOption for List {
    fn on_create_options(&mut self) {
        self.on_clear_all_var()
    }
    fn on_clear_var(&mut self) {
        //if 'Filldown' not in self.value.OptionNames():
        //self._value = []
        self.0.clear()
    }
    fn on_clear_all_var(&mut self) {
        self.0.clear()
    }
    fn on_assign_var(&mut self) {
        let matches: Option<HashMap<String, String>> = match self.1.value {
            Some(v) => match self.1.compiled_regex.captures(&v) {
                Some(caps) => Some(
                    self.1
                        .compiled_regex
                        .capture_names()
                        .flatten()
                        .filter_map(|n| Some((n.to_string(), caps.name(n)?.as_str().to_string())))
                        .collect(),
                ),
                None => None,
            },
            None => None,
        };

        if let Some(m) = matches {
            if m.len() > 1 {
                self.0.push(Left(m.to_owned()));
            } else {
                match self.1.value {
                    Some(v) => self.0.push(Right(v)),
                    None => {}
                }
            }
        }
    }
    fn on_get_value(&mut self) {}
    fn on_save_record(&mut self) -> Result<StateChangeTrigger, ()> {
        self.1.values = self.0.clone();
        Ok(StateChangeTrigger::Continue)
    }
}
