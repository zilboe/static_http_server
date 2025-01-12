use serde::{Serializer, Deserializer};
use std::{fs, env};
struct HttpConfig<'a> {
    ip_port: Option<&'a str>,
    web_page: Option<&'a str>,
}

impl<'a> HttpConfig<'a> {
    pub fn new() -> Self {
        HttpConfig {
            ip_port: None,
            web_page: None,
        }
    }

    fn open_config() -> Result<&'a str, ()> {
        let env_args: Vec<String> = env::args().collect();
        if env_args.len() < 2 {
            println!("Please Run <{}> With Config",env_args[0]);
            return Err(())
        }
        let config_file = env_args[1];

        let config_content = match fs::read_to_string(config_file) {
            Ok(config) => config,
            Err(_) => {
                println!("Can't Open Read The Config");
                return Err(())
            }
        };
    }

    fn get_config(&mut self) -> Result<Self, ()> {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {

    }
}
