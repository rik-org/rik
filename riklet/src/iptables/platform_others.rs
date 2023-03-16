use crate::iptables::rule::Rule;
use crate::iptables::Result;
use crate::iptables::{Iptables, IptablesError, MutateIptables};
use tracing::warn;

impl MutateIptables for Iptables {
    /// Implementation is not supported on other platform than linux
    fn create(&mut self, _: &Rule) -> Result<()> {
        warn!("Rule creation is not supported on this platform, skipping");
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }

    /// Implementation is not supported on other platform than linux
    fn delete(&mut self, _: &Rule) -> Result<()> {
        warn!("Rule deletion is not supported on this platform, skipping");
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }

    /// Implementation is not supported on other platform than linux
    fn exists(&self, _: &Rule) -> Result<bool> {
        warn!("Rules check is not supported on this platform, skipping");
        Err(IptablesError::LoadFailed(
            "Not supported on this platform".to_string(),
        ))
    }
}
