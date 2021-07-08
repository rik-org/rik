use clap::{App, Arg};
use std::error::Error;
use std::fmt;
use std::net::SocketAddrV4;

#[derive(Debug)]
pub struct ConfigParser {
    pub workers_endpoint: SocketAddrV4,
    pub controller_endpoint: SocketAddrV4,
    pub verbosity_level: String,
}

#[derive(Debug)]
pub enum ConfigParserError {
    InvalidWorkersEndpoint,
    InvalidControllersEndpoint,
}

impl ConfigParser {
    pub fn new() -> Result<ConfigParser, ConfigParserError> {
        let matches = App::new("RIK scheduler")
            .version("1.0")
            .author("Polytech Montpellier - DO3 - 2023")
            .arg(
                Arg::with_name("workers_ip")
                    .short("wip")
                    .long("workersip")
                    .value_name("WORKERS_IP")
                    .help("Workers endpoint IPv4")
                    .takes_value(true)
                    .default_value("0.0.0.0:4995"),
            )
            .arg(
                Arg::with_name("controllers_ip")
                    .short("cip")
                    .long("ctrlip")
                    .value_name("CONTROLLERS_IP")
                    .help("Controllers endpoint IPv4")
                    .takes_value(true)
                    .default_value("0.0.0.0:4996"),
            )
            .arg(
                Arg::with_name("v")
                    .short("v")
                    .multiple(true)
                    .help("Sets the level of verbosity"),
            )
            .get_matches();

        let workers_ip: SocketAddrV4 = matches
            .value_of("workers_ip")
            .unwrap()
            .parse()
            .map_err(|_| ConfigParserError::InvalidWorkersEndpoint)?;

        let controllers_ip: SocketAddrV4 = matches
            .value_of("controllers_ip")
            .unwrap()
            .parse()
            .map_err(|_| ConfigParserError::InvalidControllersEndpoint)?;

        Ok(ConfigParser {
            workers_endpoint: workers_ip,
            controller_endpoint: controllers_ip,
            verbosity_level: ConfigParser::get_verbosity_level(matches.occurrences_of("v")),
        })
    }

    fn get_verbosity_level(occurrences: u64) -> String {
        String::from(match occurrences {
            0 => "info",
            1 => "debug",
            _ => "trace",
        })
    }
}

impl fmt::Display for ConfigParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ConfigParserError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_infinite() {
        let verbosity = ConfigParser::get_verbosity_level(999999);
        assert_eq!(verbosity, "trace");
    }
}
