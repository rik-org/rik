use ipnetwork::{IpNetworkError, Ipv4Network};
use std::{collections::HashMap, net::Ipv4Addr};

#[derive(Debug, Clone)]
pub struct IpAllocator {
    subnet_pool: HashMap<Ipv4Network, bool>,
}

impl IpAllocator {
    pub fn new() -> Result<IpAllocator, IpNetworkError> {
        let mut subnet_pool: HashMap<Ipv4Network, bool> = HashMap::new();
        let network = Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
        for ip in network.iter().step_by(4) {
            let subnet = Ipv4Network::new(ip, 30)?;
            subnet_pool.insert(subnet, true);
        }
        Ok(IpAllocator { subnet_pool })
    }

    pub fn allocate_subnet(&mut self) -> Option<Ipv4Network> {
        for (subnet, available) in self.subnet_pool.iter_mut() {
            if *available {
                *available = false;
                return Some(*subnet);
            }
        }
        None
    }

    pub fn free_subnet(&mut self, subnet: Ipv4Network) {
        if let Some(available) = self.subnet_pool.get_mut(&subnet) {
            *available = true;
        }
    }

    pub fn available(&self) -> usize {
        self.subnet_pool.iter().filter(|subnet| *subnet.1).count()
    }
}
