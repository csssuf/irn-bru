use actix_web::{actix::System, App, server};
use structopt::StructOpt;

use std::error::Error;
use std::path::PathBuf;

mod api;
mod auth;
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
    let auth = auth::ApiKeyAuth(config.api.api_key.clone());
    let machine = machine::Machine::from_components(config.machine);

    let system = System::new("irn-bru");

    server::new(move || App::with_state(api::ApiState::from_machine(machine.clone()))
            .middleware(auth.clone())
            .resource("/drop", |r| r.post().with(api::drop))
            .resource("/health", |r| r.get().with(api::health))
            .finish()
        )
        .bind(format!("{}:{}", opts.address, opts.port))?
        .start();

    system.run();

    Ok(())
}
