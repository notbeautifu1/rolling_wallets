mod config;
mod contract;
mod wallet;

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;
use std::env;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Wallet,
    Contract,
}

impl Cli {
    pub async fn process(self) -> Result<()> {
        match self.command {
            Command::Wallet => {
                info!("Wallet generation module selected");

                wallet::wallet().await?;
            }
            Command::Contract => {
                info!("Contract generation module selected");

                contract::contract().await?;
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "info");

    env_logger::init();

    Cli::parse().process().await?;

    Ok(())
}
