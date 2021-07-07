extern crate yaml_rust;
use crate::ApiError;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::YamlLoader;

#[derive(Debug)]
pub struct Config {}

impl Config {
    pub fn get_uri() -> Result<String, ApiError> {
        let path: String;
        match env::var("RIKCONFIG") {
            Ok(var_path) => {
                path = var_path;
            }
            Err(_) => {
                match project_root::get_project_root() {
                    //fix current dir to project root to find rik.config.yml
                    Ok(p) => {
                        assert!(env::set_current_dir(&p).is_ok());
                        path = "rik.config.yml".to_string();
                    }
                    Err(_) => return Err(ApiError::CantReadConfigFile),
                }
            }
        }
        match File::open(&path) {
            Ok(mut file) => {
                let mut contents = String::new();
                match file.read_to_string(&mut contents) {
                    Ok(_usize) => {
                        let yaml = YamlLoader::load_from_str(contents.as_str()).unwrap();
                        let uri = yaml[0]["cluster"]["server"].as_str();
                        if uri == None {
                            return Err(ApiError::BadConfigFile);
                        }
                        Ok(uri.unwrap().to_string())
                    }
                    Err(_) => Err(ApiError::CantReadConfigFile),
                }
            }
            Err(_) => Err(ApiError::CantOpenConfigFile(path.clone())),
        }
    }
}
