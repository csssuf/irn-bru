use serde::Deserialize;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct IrnBruConfig {
    pub(crate) machine: MachineConfig,
    pub(crate) api: ApiConfig,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MachineConfig {
    pub(crate) name: String,
    pub(crate) slot_addresses: Vec<String>,
    pub(crate) temp_address: String,
    pub(crate) drop_timing: u64,
    pub(crate) owfs_mountpoint: String,
}

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ApiConfig {
    pub(crate) api_key: String,
}

pub(crate) fn read_config(path: Option<PathBuf>) -> Result<IrnBruConfig, Box<dyn Error>> {
    let path = path.unwrap_or("irn-bru.toml".into());

    if !path.exists() {
        return Err(format!("couldn't find config file: {:?}", path).into());
    }

    let config_file = File::open(&path)?;
    let config_bytes = config_file.bytes().collect::<Result<Vec<_>, _>>()?;
    let config = toml::from_slice(&config_bytes)?;

    Ok(config)
}
