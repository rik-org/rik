use tracing::{error, info, trace};

use crate::iptables::rule::Rule;
use crate::iptables::Result;
use crate::iptables::{Iptables, IptablesError, MutateIptables};

impl MutateIptables for Iptables {
    /// Tries to create a rule, in case it already exists it will throw [IptablesError::AlreadyExist]
    /// Also, it will throw if your rule is invalid
    ///
    /// ## Example
    /// ```
    /// let ipt = Iptables::new(false).unwrap();
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
    fn create(&mut self, rule: &Rule) -> Result<()> {
        trace!("Tries to create iptables rule {}", rule);
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        if self.exists(rule)? {
            trace!("Could not create rule {}", rule);
            return Err(IptablesError::AlreadyExist(rule.clone()));
        }

        self.inner
            .append(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
            .and_then(|_| {
                self.rules.push(rule.clone());
                Ok(())
            })
    }
    /// Tries to delete a rule, in case it does not exist it will throw [IptablesError::AlreadyDeleted]
    /// ## Example
    /// ```
    /// let ipt = Iptables::new(false).unwrap();
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
    fn delete(&mut self, rule: &Rule) -> Result<()> {
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        if !self.exists(rule)? {
            return Err(IptablesError::AlreadyDeleted(rule.clone()));
        }
        self.inner
            .delete(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
            .and_then(|_| {
                self.rules.retain(|r| r != rule);
                Ok(())
            })
    }

    /// Tries to determine whether a rule exists or not. If it does return true, else false
    /// It might happen that iptables fails to validate the rule, in that case it will throw [IptablesError::InvalidRule]
    /// ## Example
    /// ```
    /// let ipt = Iptables::new(false).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::iptables::rule::Rule;
    use crate::iptables::Chain;
    use crate::iptables::Table;

    #[test]
    fn test_create() {
        let mut ipt = Iptables::new(false).unwrap();
        let rule = Rule::new(
            Chain::Input,
            Table::Filter,
            "-p tcp --dport 80 -j ACCEPT".to_string(),
        );
        let result = ipt.create(&rule);
        assert!(result.is_ok());
        let result = ipt.create(&rule);
        assert!(result.is_err());
        let result = ipt.delete(&rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete() {
        let mut ipt = Iptables::new(false).unwrap();
        let rule = Rule::new(
            Chain::Input,
            Table::Filter,
            "-p tcp --dport 444 -j ACCEPT".to_string(),
        );
        let result = ipt.create(&rule);
        assert!(result.is_ok());
        let result = ipt.delete(&rule);
        assert!(result.is_ok());
        let result = ipt.delete(&rule);
        assert!(result.is_err());
    }

    #[test]
    fn test_exists() {
        let mut ipt = Iptables::new(false).unwrap();
        let rule = Rule::new(
            Chain::Input,
            Table::Filter,
            "-p tcp --dport 443 -j ACCEPT".to_string(),
        );
        let result = ipt.create(&rule);
        assert!(result.is_ok());
        let result = ipt.exists(&rule);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
        ipt.delete(&rule).unwrap();
        let result = ipt.exists(&rule);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }
}
