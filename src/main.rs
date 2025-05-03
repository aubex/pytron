use pytron::{Cli, Commands};
use clap::Parser;
use std::process::exit;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Zip { directory, output, ignore_patterns } => {
            if let Err(err) = pytron::zip_directory(&directory, &output, ignore_patterns.as_ref()) {
                eprintln!("Error zipping directory: {}", err);
                exit(1);
            }
        }
        Commands::Run {
            zipfile,
            script,
            args,
        } => {
            let exit_code = match pytron::run_from_zip(&zipfile, &script, args) {
                Ok(code) => code,
                Err(err) => {
                    eprintln!("Error running from zip: {}", err);
                    1
                }
            };
            exit(exit_code);
        }
    }
}