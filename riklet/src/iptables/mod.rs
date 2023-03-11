use iptables::IPTables as LibIptables;
use std::convert::TryFrom;
use std::fmt::Display;
use thiserror::Error;

/// A wrapper around original iptables crates in order to better match
/// rust usages. You can use this crate directly or use the wrapper
///
/// ```
/// use crate::iptables::Iptables;
/// let iptables = Iptables::new();
/// ```
pub struct Iptables {
    inner: LibIptables,
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
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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

pub trait MutateIptables {
    fn create(&self, rule: &Rule) -> Result<()>;
    fn delete(&self, rule: &Rule) -> Result<()>;
    fn exists(&self, rule: &Rule) -> Result<bool>;
}

#[cfg(target_os = "linux")]
impl MutateIptables for Iptables {
    /// Tries to create a rule, in case it already exists it will throw [IptablesError::AlreadyExist]
    /// Also, it will throw if your rule is invalid
    ///
    /// ## Example
    /// ```
    /// let ipt = Iptables::new().unwrap();
    /// let rule = Rule::new(Table::Filter, Chain::Input, "-p tcp --dport 80 -j ACCEPT".to_string());
    /// let result = ipt.create(&rule);
    /// assert!(result.is_ok());
    ///
    /// let result = ipt.create(&rule);
    /// assert!(result.is_err());
    ///
    /// let result = ipt.delete(&rule);
    /// assert!(result.is_ok());
    /// ```
    fn create(&self, rule: &Rule) -> Result<()> {
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        if self.exists(rule)? {
            return Err(IptablesError::AlreadyExist(rule.clone()));
        }
        self.inner
            .append(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
    }
    /// Tries to delete a rule, in case it does not exist it will throw [IptablesError::AlreadyDeleted]
    /// ## Example
    /// ```
    /// let ipt = Iptables::new().unwrap();
    /// let rule = Rule::new(Table::Filter, Chain::Input, "-p tcp --dport 80 -j ACCEPT".to_string());
    /// let result = ipt.create(&rule);
    /// assert!(result.is_ok());
    ///
    /// let result = ipt.delete(&rule);
    /// assert!(result.is_ok());
    ///
    /// let result = ipt.delete(&rule);
    /// assert!(result.is_err());
    /// ```
    fn delete(&self, rule: &Rule) -> Result<()> {
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        if !self.exists(rule)? {
            return Err(IptablesError::AlreadyDeleted(rule.clone()));
        }
        self.inner
            .delete(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
    }

    /// Tries to determine whether a rule exists or not. If it does return true, else false
    /// It might happen that iptables fails to validate the rule, in that case it will throw [IptablesError::InvalidRule]
    /// ## Example
    /// ```
    /// let ipt = Iptables::new().unwrap();
    /// let rule = Rule::new(Table::Filter, Chain::Input, "-p tcp --dport 80 -j ACCEPT".to_string());
    /// let result = ipt.create(&rule);
    /// assert!(result.is_ok());
    /// let result = ipt.exists(&rule);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), true);
    ///
    /// ipt.delete(&rule).unwrap();
    /// let result = ipt.exists(&rule);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), false);
    fn exists(&self, rule: &Rule) -> Result<bool> {
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        self.inner
            .exists(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|_| IptablesError::InvalidRule(rule.clone()))
    }
}

#[cfg(not(target_os = "linux"))]
impl MutateIptables for Iptables {
    fn create(&self, _rule: &Rule) -> Result<()> {
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }

    fn delete(&self, _rule: &Rule) -> Result<()> {
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }

    fn exists(&self, _rule: &Rule) -> Result<bool> {
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }
}

impl Iptables {
    #[cfg(target_os = "linux")]
    /// Create a new instance of Iptables manager, it will allow to manage your iptable
    /// chains and rules
    pub fn new() -> Result<Self> {
        iptables::new(false)
            .map(|iptables| Iptables { inner: iptables })
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
    }

    #[cfg(not(target_os = "linux"))]
    pub fn new() -> Result<Self> {
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

impl From<LibIptables> for Iptables {
    fn from(iptables: LibIptables) -> Iptables {
        Iptables { inner: iptables }
    }
}

#[derive(Debug, Clone)]
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iptables_new() {
        let iptables = Iptables::new();
        assert!(iptables.is_ok());
    }
}
