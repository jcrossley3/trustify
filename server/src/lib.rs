#![allow(unused)]

use crate::server::read;
use actix_web::web;
use huevos_api::system::System;
use std::process::ExitCode;

pub mod server;

/// Run the API server
#[derive(clap::Args, Debug)]
pub struct Run {}

impl Run {
    pub async fn run(self) -> anyhow::Result<ExitCode> {
        todo!();

        Ok(ExitCode::SUCCESS)
    }
}

pub struct AppState {
    pub system: System,
}

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(read::package::dependencies)
        .service(read::package::variants);
}

#[cfg(test)]
mod test_util {
    use huevos_api::system::System;

    pub async fn bootstrap_system(name: &str) -> Result<System, anyhow::Error> {
        System::bootstrap("postgres", "eggs", "localhost", name).await
    }
}