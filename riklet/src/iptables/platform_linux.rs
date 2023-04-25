use crate::iptables::rule::Rule;
use crate::iptables::Result;
use crate::iptables::{trace, Iptables, IptablesError, MutateIptables};

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
            .map(|_| self.rules.push(rule.clone()))
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
        trace!("Tries to delete iptables rule {}", rule);
        self.validate_combo_table_chain(rule.table.clone(), rule.chain.clone())?;
        if !self.exists(rule)? {
            trace!("Could not delete rule {}", rule);
            return Err(IptablesError::AlreadyDeleted(rule.clone()));
        }
        self.inner
            .delete(&rule.table.to_string(), &rule.chain.to_string(), &rule.rule)
            .map_err(|e| IptablesError::LoadFailed(e.to_string()))
            .map(|_| self.rules.retain(|r| r != rule))
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

    fn create_chain(&mut self, chain: &super::Chain, table: &super::Table) -> Result<()> {
        if !chain.is_custom() {
            return Ok(());
        }

        self.chains.push((table.clone(), chain.clone()));

        self.inner
            .new_chain(table.to_string().as_str(), chain.to_string().as_str())
            .map_err(|e| IptablesError::InvalidChain(e.to_string()))
    }

    fn delete_chain(&mut self, chain: &super::Chain, table: &super::Table) -> Result<()> {
        if !chain.is_custom() {
            return Ok(());
        }

        self.chains.retain(|(t, c)| t != table || c != chain);

        self.inner
            .delete_chain(table.to_string().as_str(), chain.to_string().as_str())
            .map_err(|e| IptablesError::InvalidChain(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::iptables::rule::Rule;
    use crate::iptables::Chain;
    use crate::iptables::Table;

    #[test]
    #[serial]
    fn test_create_chain_default() {
        let mut ipt = Iptables::new(true).unwrap();
        let result = ipt.create_chain(&Chain::Input, &Table::Filter);
        assert!(result.is_ok());
        let result = ipt.create_chain(&Chain::Input, &Table::Filter);
        assert!(result.is_ok());
        let result = ipt.delete_chain(&Chain::Input, &Table::Filter);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_create_chain_custom() {
        let mut ipt = Iptables::new(false).unwrap();
        let result = ipt.create_chain(&Chain::Custom("test".to_string()), &Table::Filter);
        assert!(result.is_ok());
        let result = ipt.create_chain(&Chain::Custom("test".to_string()), &Table::Filter);
        assert!(result.is_err());
        let result = ipt.delete_chain(&Chain::Custom("test".to_string()), &Table::Filter);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_chain_drop() {
        {
            let mut ipt_dopped = Iptables::new(true).unwrap();
            let result =
                ipt_dopped.create_chain(&Chain::Custom("test002".to_string()), &Table::Filter);
            assert!(result.is_ok());
        }
        let ipt = Iptables::new(false).unwrap();
        let res = ipt.inner.chain_exists("test002", "filter").unwrap();
        assert!(!res);
    }

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
