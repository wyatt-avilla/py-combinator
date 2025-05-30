mod deserialize;
mod serialize;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Argument {
    pub mutable: bool,
    pub name: String,
    pub expected_type: String,
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mutable {
            write!(f, "mut {}: {}", self.name, self.expected_type)
        } else {
            write!(f, "{}: {}", self.name, self.expected_type)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Method {
    pub comments: Option<String>,
    pub name: String,
    pub args: Vec<Argument>,
    pub return_type: Option<String>,
    pub literal_return: bool,
    pub strips: Vec<String>,
}
