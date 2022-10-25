#[doc(hidden)]
mod commands;
#[doc(hidden)]
mod errors;
#[doc(hidden)]
mod parser;
#[doc(hidden)]
mod report_printer;
mod vcs_manager;

use clap::Parser;
use parser::{Cli, Commands};

fn main() {
    let cli = Cli::parse();
    let result = match &cli.command {
        Commands::Init { path } => commands::init::run(path),
        Commands::Commit { message } => commands::commit::run(message),
        Commands::Log => commands::log::run(),
        Commands::Status => commands::status::run(),
        Commands::Merge { branch } => commands::merge::run(branch),
        Commands::NewBranch { name } => commands::new_branch::run(name),
        Commands::Jump { branch, commit } => {
            if let Some(branch_name) = branch {
                commands::jump::to_branch(branch_name)
            } else if let Some(id) = commit {
                commands::jump::to_commit(id)
            } else {
                unreachable!()
            }
        }
    };
    match result {
        Ok(report) => {
            print!("{report}");
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}
