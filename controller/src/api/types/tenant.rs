use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub value: String,
}

impl fmt::Display for Tenant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id: {}, Name: {}", self.id, self.name)
    }
}
