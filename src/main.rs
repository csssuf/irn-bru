use actix_web::{App, http, server};
use structopt::StructOpt;

use std::error::Error;
use std::path::PathBuf;

mod api;
mod config;
mod machine;

#[derive(Debug, StructOpt)]
#[structopt(name = "irn-bru", about = "irn-bru CSH drink machine server")]
struct IrnBruCli {
    /// Path to irn-bru config file
    #[structopt(short = "c", long = "config", parse(from_os_str))]
    config_path: Option<PathBuf>,
    /// Address for API server to listen on
    #[structopt(short = "a", long = "address", default_value = "0.0.0.0")]
    address: String,
    /// Port for API server to listen on
    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u16,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts = IrnBruCli::from_args();

    let config = config::read_config(opts.config_path)?;
    let machine = machine::Machine::from_components(config.machine);

    server::new(move || App::with_state(api::ApiState::from_machine(machine.clone()))
            .resource("/drop", |r| r.post().with(api::drop))
            .finish()
        )
        .bind(format!("{}:{}", opts.address, opts.port))?
        .start();

    Ok(())
}
