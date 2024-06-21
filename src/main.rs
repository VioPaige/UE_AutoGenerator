#![allow(non_snake_case)]
mod lib {
    pub mod input;
    pub mod output;
    pub mod config;
    pub mod fs;
    pub mod process;
}

use std::path::PathBuf;
#[allow(unused_imports)]
use lib::{
    output::colouriseLog,
    input::parseArguments, 
    config::{
        Config,
        checkConfig
    },
    process::{
        lookForMatch,
        generateTarget,
    }
};



fn main() {
    let args: Vec<String> = std::env::args()
        .filter(|arg| !arg.ends_with(".exe"))
        .into_iter()
        .collect();
    let parsedArgs = parseArguments(args);

    let configPath: PathBuf;
    if let Some(p) = parsedArgs.get("config") {
        configPath = PathBuf::from(p);
    } else {
        configPath = PathBuf::from("config.json");
    }

    let config: Config = checkConfig(configPath, parsedArgs.clone());

    let mode = parsedArgs.get("mode").expect("'mode' argument is a required argument.");
    if mode == "lfm" || mode == "lookformatch" {
        lookForMatch(config, parsedArgs);
    } else if mode == "targeted" || mode == "t" || mode == "td" || mode == "targeteddir" {
        generateTarget(config, parsedArgs);
    }
}
