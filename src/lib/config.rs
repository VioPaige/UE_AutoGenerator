extern crate serde_derive;
extern crate sysinfo;

use std::{
    fs::File, 
    path::PathBuf,
    collections::HashMap
};

use self::serde_derive::Deserialize;
use self::sysinfo::System;



#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct FileConfig {
    pub ue4cmd: Option<PathBuf>,
    pub project: Option<PathBuf>,
    pub projectdir: Option<PathBuf>,
    pub dumpdir: Option<PathBuf>,
    pub nongendump: Option<PathBuf>,

    pub assetOrder: Option<Vec<String>>,
    pub refreshExistingAssets: Option<bool>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Config {
    pub fileConfig: FileConfig,

    pub maxBatchSize: u64,
}

pub fn checkConfig(path: PathBuf, overrideArguments: HashMap<String, String>) -> Config {
    let file = File::open(path).expect("Could not open config file.");
    let fileConfig: FileConfig = serde_json::from_reader(file).expect("Failed to read config file as json.");

    if fileConfig.ue4cmd.is_none() || fileConfig.project.is_none() || fileConfig.projectdir.is_none() || fileConfig.dumpdir.is_none() || fileConfig.nongendump.is_none() {
        println!("Invalid json, make sure all required fields are included.");
        std::process::exit(0);
    }

    let mut system = System::new_all();
    system.refresh_all();

    let config: Config = Config {
        fileConfig,
        maxBatchSize: if let Some(maxBatchSize) = overrideArguments.get("maxBatchSize") { maxBatchSize.parse::<u64>().unwrap() } else { system.total_memory() / (1_000_000_000) * 90 }
    };

    return config;
}