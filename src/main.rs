mod cli;
mod commands;
mod constants;
mod display;
mod error;
mod models;
mod services;
mod tui;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::execute(),
        Commands::Create { json } => commands::create::execute(json),
        Commands::Validate { name } => commands::validate::execute(name),
        Commands::List { tree, json } => commands::list::execute(tree, json),
        Commands::Start { name } => commands::start::execute(name),
        Commands::Done { name } => commands::done::execute(name),
        Commands::Merged { name } => commands::merged::execute(name),
        Commands::Archive { name } => commands::archive::execute(name),
        Commands::Next { json } => commands::next::execute(json),
        Commands::Reset { name } => commands::reset::execute(name),
        Commands::Status { json, action, task } => commands::status::execute(json, action, task),
        Commands::Tail { name, count } => commands::tail::execute(name, count),
        Commands::Logs => commands::logs::execute(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
