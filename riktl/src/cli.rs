extern crate clap;

use clap::{App, Arg};

#[derive(Debug)]
pub struct Cli {
    pub action: Action,
    pub entity: Entity,
    pub workload_id: String,
    pub instance_id: String,
    pub file: String,
}

#[derive(Debug)]
pub enum CliError {
    MissingArg(String),
}

#[derive(Debug, PartialEq)]
pub enum Action {
    CREATE,
    DELETE,
    GET,
}

#[derive(Debug, PartialEq)]
pub enum Entity {
    INSTANCE,
    WORKLOAD,
}

const CMD_CREATE: &'static str = "create";
const CMD_DELETE: &'static str = "delete";
const CMD_GET: &'static str = "get";
const CMD_INSTANCE: &'static str = "instance";
const CMD_WORKLOAD: &'static str = "workload";

impl Cli {
    pub fn new() -> Result<Self, CliError> {
        let args = App::new("riktl")
            .author("Polytech Montpellier - DO 2023")
            .version(&*format!("{}{}", "v", clap::crate_version!()))
            .about("RIK Command Line Interface - A rustlang based cloud orchestrator")
            .arg(
                Arg::new("action")
                    .required(true)
                    .about("The action to perform")
                    .possible_values(&[CMD_CREATE, CMD_DELETE, CMD_GET])
                    .index(1),
            )
            .arg(
                Arg::new("entity")
                    .required(true)
                    .about("The entity to handle")
                    .possible_values(&[CMD_INSTANCE, CMD_WORKLOAD])
                    .index(2),
            )
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .about("The YAML file containing the workload description")
                    .takes_value(true)
                    .required_if_eq_all(&[("action", CMD_CREATE), ("entity", CMD_WORKLOAD)]),
            )
            .arg(
                Arg::new("workload_id")
                    .short('w')
                    .long("workload")
                    .about("The target workload id")
                    .takes_value(true)
                    .required_if_eq_all(&[("action", CMD_CREATE), ("entity", CMD_INSTANCE)]),
            )
            .arg(
                Arg::new("instance_id")
                    .short('i')
                    .long("instance")
                    .about("The target instance id")
                    .takes_value(true)
                    .required_if_eq_all(&[("action", CMD_DELETE), ("entity", CMD_INSTANCE)]),
            )
            .get_matches();

        let action = match args
            .value_of("action")
            .expect("Missing action argument (create | delete | get)")
        {
            CMD_CREATE => Action::CREATE,
            CMD_DELETE => Action::DELETE,
            CMD_GET => Action::GET,
            _ => panic!(),
        };

        let entity = match args
            .value_of("entity")
            .expect("Missing entity argument (workload | instance")
        {
            CMD_WORKLOAD => Entity::WORKLOAD,
            CMD_INSTANCE => Entity::INSTANCE,
            _ => panic!(),
        };

        let file: &str;
        match args.value_of("file") {
            Some(f) => file = f,
            None => file = "",
        }

        let workload_id: &str;
        match args.value_of("workload_id") {
            Some(i) => workload_id = i,
            None => {
                if action == Action::DELETE && entity == Entity::WORKLOAD {
                    return Err(CliError::MissingArg(String::from("workload_id")));
                } else {
                    workload_id = ""
                }
            }
        }

        let instance_id: &str;
        match args.value_of("instance_id") {
            Some(i) => instance_id = i,
            None => {
                if action == Action::DELETE && entity == Entity::INSTANCE {
                    return Err(CliError::MissingArg(String::from("instance_id")));
                } else {
                    instance_id = ""
                }
            }
        }
        Ok(Cli {
            action,
            entity: entity,
            file: String::from(file),
            workload_id: String::from(workload_id),
            instance_id: String::from(instance_id),
        })
    }
}
