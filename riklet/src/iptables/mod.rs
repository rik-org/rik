#[cfg(target_os = "linux")]
pub mod platform_linux;
#[cfg(not(target_os = "linux"))]
pub mod platform_others;
pub mod rule;

use crate::iptables::rule::Rule;
use iptables::IPTables as LibIptables;
use std::convert::TryFrom;
use std::fmt::Display;
use thiserror::Error;
use tracing::{error, trace};

/// A wrapper around original iptables crates in order to better match
/// rust usages. You can use this crate directly or use the wrapper
///
/// ```
/// use crate::iptables::Iptables;
/// let iptables = Iptables::new(false);
/// ```
pub struct Iptables {
    /// Wrapped object
    inner: LibIptables,
    /// If true, the iptables will be flushed when the object is dropped (default: false)
    cleanup: bool,
    rules: Vec<Rule>,
    /// When using custom chains, we want to store created chains in
    /// order to be able to flush them
    chains: Vec<(Table, Chain)>,
}

#[derive(Debug, Error)]
pub enum IptablesError {
    #[error("Could not load iptables: {0}")]
    LoadFailed(String),
    #[error("Chain or table in rule '{0}' could not be found")]
    InvalidRule(Rule),
    #[error("Given table '{0}' is not valid")]
    InvalidTable(String),
    #[error("Given combo table '{table}' and chain '{chain}' is not valid")]
    InvalidCombo { table: Table, chain: Chain },
    #[error("Rule '{0}' already exists")]
    AlreadyExist(Rule),
    #[error("Rule '{0}' does not exist")]
    AlreadyDeleted(Rule),
    #[error("Could not manage given chain '{0}'")]
    InvalidChain(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Default list of chain available in iptables
/// Custom chain can also be used by using [Chain::Custom]
pub enum Chain {
    Input,
    Output,
    Forward,
    PostRouting,
    PreRouting,
    Custom(String),
}

impl From<String> for Chain {
    fn from(value: String) -> Self {
        match value.as_str() {
            "INPUT" => Chain::Input,
            "OUTPUT" => Chain::Output,
            "FORWARD" => Chain::Forward,
            "POSTROUTING" => Chain::PostRouting,
            "PREROUTING" => Chain::PreRouting,
            _ => Chain::Custom(value),
        }
    }
}

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Input => write!(f, "INPUT"),
            Chain::Output => write!(f, "OUTPUT"),
            Chain::Forward => write!(f, "FORWARD"),
            Chain::PostRouting => write!(f, "POSTROUTING"),
            Chain::PreRouting => write!(f, "PREROUTING"),
            Chain::Custom(value) => write!(f, "{}", value),
        }
    }
}

impl Chain {
    pub fn is_custom(&self) -> bool {
        match self {
            Chain::Custom(_) => true,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Default list of table available in iptables
pub enum Table {
    Filter,
    Nat,
    Mangle,
    Raw,
}

impl TryFrom<String> for Table {
    type Error = IptablesError;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "filter" => Ok(Table::Filter),
            "nat" => Ok(Table::Nat),
            "mangle" => Ok(Table::Mangle),
            "raw" => Ok(Table::Raw),
            _ => Err(IptablesError::InvalidTable(value)),
        }
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Table::Filter => write!(f, "filter"),
            Table::Nat => write!(f, "nat"),
            Table::Mangle => write!(f, "mangle"),
            Table::Raw => write!(f, "raw"),
        }
    }
}

type Result<T> = std::result::Result<T, IptablesError>;

/// A common interface in order to manage iptables rules, Iptables is only available on linux,
/// this interface makes able to develop on other platform
pub trait MutateIptables {
    fn create(&mut self, rule: &Rule) -> Result<()>;
    fn create_chain(&mut self, chain: &Chain, table: &Table) -> Result<()>;
    fn delete(&mut self, rule: &Rule) -> Result<()>;
    fn delete_chain(&mut self, chain: &Chain, table: &Table) -> Result<()>;
    fn exists(&self, rule: &Rule) -> Result<bool>;
}

impl Drop for Iptables {
    fn drop(&mut self) {
        if self.cleanup {
            let rules = self.rules.clone();
            for rule in rules.iter() {
                trace!("Drop iptables rule {}", rule);
                self.delete(rule).unwrap_or_else(|e| {
                    error!("Could not delete rule '{:?}', reason: {}", rule, e);
                });
            }

            let chains = self.chains.clone();
            for (table, chain) in chains.iter() {
                trace!("Drop iptables chain {} in table {}", chain, table);

                self.delete_chain(chain, table).unwrap_or_else(|e| {
                    error!("Could not delete chain '{:?}', reason: {}", chain, e);
                });
            }
        }
    }
}

/// [Iptables] is used to manage iptables rules on the system, you can learn
/// about more about it [here](https://linux.die.net/man/8/iptables)
///
/// We are currently using iptables to manage the traffic and expose on the
/// system workloads so they can be accessed internally and externally.
///
/// See [Iptables::new] for how to implement it
impl Iptables {
    #[cfg(target_os = "linux")]
    /// Create a new instance of Iptables manager, it will allow to manage your iptable
    /// chains and rules
    ///
    /// When giving argument cleanup to true, created iptables will be reverted once the object is dropped
    ///
    /// ```
    /// use crate::iptables::{Iptables, Rule, Chain, Table};
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Create a iptable rule
    ///     {
    ///         let mut ipt = Iptables::new(true).unwrap();
    ///         let rule = Rule {
    ///             rule: "-A INPUT -p tcp --dport 80 -j ACCEPT".to_string(),
    ///             chain: Chain::Forward,
    ///             table: Table::Filter,
    ///         };
    ///         ipt.create(&rule).unwrap();
    ///     }
    ///     // Iptables will be reverted as we are out of scope
    ///     Ok(())
    ///
    /// }
    /// ```
    pub fn new(cleanup: bool) -> Result<Self> {
        iptables::new(false)
            .map(|iptables| Iptables {
                inner: iptables,
                cleanup,
                rules: vec![],
                chains: vec![],
            })
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new(_: bool) -> Result<Self> {
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }

    #[cfg(target_os = "linux")]
    fn validate_combo_table_chain(&self, table: Table, chain: Chain) -> Result<()> {
        match self
            .inner
            .chain_exists(&table.to_string(), &chain.to_string())
        {
            Ok(_) => Ok(()),
            Err(_) => Err(IptablesError::InvalidCombo { table, chain }),
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn validate_combo_table_chain(&self, table: Table, chain: Chain) -> Result<()> {
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iptables_new() {
        let iptables = Iptables::new(false);
        assert!(iptables.is_ok());
    }

    #[test]
    fn iptables_drop_clean() {
        let mut iptables = Iptables::new(true).unwrap();
        let rule = Rule::new(
            Chain::Input,
            Table::Filter,
            "-p tcp --dport 440 -j ACCEPT".to_string(),
        );
        iptables.create(&rule).unwrap();
        assert!(iptables.exists(&rule).unwrap());
        drop(iptables);
        let iptables = Iptables::new(false).unwrap();
        assert!(!iptables.exists(&rule).unwrap());
    }

    #[test]
    fn iptables_clean_disabled_doesnt_drop() {
        let mut iptables = Iptables::new(false).unwrap();
        let rule = Rule::new(
            Chain::Input,
            Table::Filter,
            "-p tcp --dport 442 -j ACCEPT".to_string(),
        );
        iptables.create(&rule).unwrap();
        assert!(iptables.exists(&rule).unwrap());
        drop(iptables);
        let mut iptables = Iptables::new(false).unwrap();
        assert!(iptables.exists(&rule).unwrap());
        iptables.delete(&rule).unwrap();
    }
}
