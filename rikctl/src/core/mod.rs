use prettytable::{format, Table};

pub mod client;
pub mod config;
pub mod instance;
pub mod workload;

pub fn get_display_table() -> Table {
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_CLEAN);
    table
}
