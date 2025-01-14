use std::{fs, env};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Config {
    ip_port: Option<String>,
    web_port: Option<String>
}

impl Config {
    pub fn read_config() -> Self {
        let configs = configs().unwrap();
        Self { 
            ip_port: configs.ip_port, 
            web_port: configs.web_port,
        }
    }
}

fn configs() -> Result<Config, ()> {
    let env_arg: Vec<String> = env::args().collect();
    if env_arg.len() < 2 {
        println!("Please Run The ({}) With Config", env_arg[0]);
        return Err(())
    }
    let config_content_res = fs::read_to_string(&env_arg[1]);
    let config_content = match config_content_res {
        Ok(config) => config,
        Err(_) => {
            println!("Can't Read The Config");
            return Err(())
        }
    };
    
    let configs = match serde_json::from_str(&config_content) {
        Ok(configs) => configs,
        Err(_) => {
            println!("Can't Serialize The File Config");
            return Err(())
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
