
use std::{fs, env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub ip_port: String,
    pub web_page: String
}

struct ConfigError<'a> {
    message: &'a str
}

impl Config {
    pub fn read_config() -> Result<Self, ()> {
        match configs() {
            Ok(configs) => {
                Ok(configs)
            },
            Err(e) => {
                println!("{}", e.message);
                return Err(())
            }
        }
    }
}

fn configs() -> Result<Config, ConfigError<'static>> {
    let env_arg: Vec<String> = env::args().collect();
    if env_arg.len() < 2 {
        let err = "Please Run The With Config";
        return Err(
            ConfigError { 
                message: err
         })
    }
    let config_content_res = fs::read_to_string(&env_arg[1]);
    let config_content = match config_content_res {
        Ok(config) => config,
        Err(_) => {
            let err = "Can't Read The Config";
            return Err(
                ConfigError { 
                    message: err
             })
        }
    };
    
    let configs = match serde_json::from_str(&config_content) {
        Ok(configs) => configs,
        Err(_) => {
            let err = "Can't Serialize The File Config";
            return Err(
                ConfigError { 
                    message: err
             })
        }
    };
    Ok(configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let configs = Config::read_config();
        
        println!("{:?}", configs.ip_port);
        println!("{:?}", configs.web_port);
    }
}
