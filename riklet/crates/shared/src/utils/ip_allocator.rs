use ipnetwork::Ipv4Network;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct IpAllocator {
    subnet_pool: HashMap<Ipv4Network, bool>,
}

impl IpAllocator {
    pub fn new(network: Ipv4Network) -> IpAllocator {
        let mut subnet_pool: HashMap<Ipv4Network, bool> = HashMap::new();
        for ip in network.iter().step_by(4) {
            let subnet = Ipv4Network::new(ip, 30).unwrap();
            subnet_pool.insert(subnet, true);
        }
        IpAllocator { subnet_pool }
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
