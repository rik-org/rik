use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub workload_id: String,
    pub status: u16,
}
