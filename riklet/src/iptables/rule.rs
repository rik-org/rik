use crate::iptables::{Chain, Table};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Define an Iptable rule, this object can't be able to determine whether the rule is valid, you'll
/// only be able to know it when you are running [crate::iptables::Iptables::create] or other methods
pub struct Rule {
    pub chain: Chain,
    pub table: Table,
    pub rule: String,
}

impl Rule {
    pub fn new(chain: Chain, table: Table, rule: String) -> Self {
        Rule { chain, table, rule }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({}): {}",
            self.table.to_string(),
            self.chain.to_string(),
            self.rule
        )
    }
}
