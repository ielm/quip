use clap::{Parser, Subcommand};
use cliclack::{intro, outro};
// use cliclack::{intro, note, outro};
use quip::consts::TITLE_TEXT;
use quip::prelude::*;

/// The Quip CLI
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a base project
    Init(InitCommand),

    /// Pull a problem from LeetCode
    Pull(PullCommand),
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    // println!("{}", TITLE_TEXT);
    intro(TITLE_TEXT).expect("Could not print intro");
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(init) => {
            init.run();
        }
        Commands::Pull(pull) => {
            pull.run().await;
        }
    }
    outro("Good luck on your journey.\n").expect("Could not print outro");
}
