mod bluez;
mod bt;
mod bt_parsing;
mod emitters;
mod event;
mod ibeacon_parsing;

use anyhow::Result;
use clap::Clap;
use emitters::{Emitter, Emitters};
use event::Dispatcher;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use tracing::error;

#[macro_use]
extern crate num_derive;

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Robin Sonefors <robin@sonefors.net")]
struct Opts {
    #[clap(short, long)]
    config: String,
    #[clap(short, long, parse(from_occurrences))]
    verbosity: i32,
}

#[derive(Deserialize, Debug)]
struct Config {
    #[serde(flatten)]
    emitters: HashMap<String, Emitters>,
}

fn load(config_str: &str) -> Result<Vec<Box<dyn Emitter>>> {
    let config: Config = toml::from_str(&config_str)?;
    let modules = emitters::init(&config.emitters)?;
    Ok(modules)
}

fn main() -> Result<()> {
    env_logger::init();
    let opts: Opts = Opts::parse();
    let config_str = read_to_string(opts.config)?;
    let modules = load(&config_str);
    if let Err(e) = modules {
        error!("There was an error parsing the config: {}", e);
        return Err(e);
    }
    let modules = modules.unwrap();
    let dispatcher = Dispatcher { modules };
    bt::run(&dispatcher)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_config() {
        let modules = load(r#""#);
        assert!(modules.is_ok());
        assert_eq!(modules.unwrap().len(), 0);
    }

    #[test]
    fn log_config() -> Result<(), Box<dyn std::error::Error>> {
        let modules = load(
            r#"[log]
emitter = "log"
"#,
        );
        modules?;
        Ok(())
    }

    #[test]
    fn every_config() -> Result<(), Box<dyn std::error::Error>> {
        let modules = load(
            r#"[log]
emitter = "log"

[http]
emitter = "http"
url = "http://foo"
[http.payload]

[prometheus]
emitter = "prometheus"
address = "foo"
temp_gauge_name = "temp_foo"
gravity_gauge_name = "gravity_foo"
"#,
        )?;
        assert!(modules.len() == 3);
        Ok(())
    }

    #[test]
    fn multiple_config() -> Result<(), Box<dyn std::error::Error>> {
        let modules = load(
            r#"[brewservice1]
emitter = "http"
url = "http://foo"
payload = {}
[brewservice2]
emitter = "http"
url = "http://bar"
payload = {}
"#,
        )?;
        assert!(modules.len() == 2);
        Ok(())
    }
}
