extern crate clap;

use clap::{App, Arg};

#[derive(Debug)]
pub struct Cli {
    args: Vec<String>,
}

const CMD_CREATE: &'static str = "create";
const CMD_DELETE: &'static str = "delete";
const CMD_GET: &'static str = "get";
const CMD_INSTANCE: &'static str = "instance";
const CMD_WORKLOAD: &'static str = "workload";

impl Cli {

    pub fn new() -> Self {
        let _clap = App::new("riktl")
        .author(clap::crate_authors!())
        .version(&*format!("{}{}", "v", clap::crate_version!()))
        .about("RIK Command Line Interface - A rustlang based cloud orchestrator")
        .arg(
            Arg::new("action")
            .required(true)
            .about("The action to perform")
            .possible_values(&[CMD_CREATE, CMD_DELETE, CMD_GET])
            .index(1))
        .arg(
            Arg::new("entity")
            .required(true)
            .about("The entity to handle")
            .possible_values(&[CMD_INSTANCE, CMD_WORKLOAD])
            .index(2))
        .arg(
            Arg::new("file")
            .short('f')
            .long("file")
            .about("The YAML file containing the workload description")
            .takes_value(true)
            .required_if_eq_all(&[("action", CMD_CREATE), ("entity", CMD_WORKLOAD)]))
        .arg(Arg::new("workload_id")
            .short('w')
            .long("workload")
            .about("The target workload id")
            .takes_value(true)
            .required_if_eq_all(&[("action", CMD_CREATE), ("entity", CMD_INSTANCE)]))
        .arg(Arg::new("instance_id")
            .short('i')
            .long("instance")
            .about("The target instance id")
            .takes_value(true)
            .required_if_eq_all(&[("action", CMD_DELETE), ("entity", CMD_INSTANCE)]))
        .get_matches();
        
        
        let args = Vec::new();
        Cli{args}
    }
}