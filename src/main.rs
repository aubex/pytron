use clap::Parser;
use pytron::{Cli, Commands};
use std::{env, process::exit};
use dotenv::dotenv;

fn main() {
    // On Windows, check for long path support at startup
    #[cfg(windows)]
    {
        dotenv().ok();
        match pytron::check_and_enable_long_path_support() {
            Ok(true) => {
                // Long path support is enabled, continue normally
            }
            Ok(false) => {
                println!("Long path support has been enabled, but you need to reboot for it to take effect.");
                println!("After rebooting, run this command again.");
            }
            Err(e) => {
                println!("Warning: Could not check or enable long path support: {}", e);
                println!("You may encounter issues with long file paths.");
            }
        }
    }
    
    let args: Vec<String> = env::args().collect();

    // Check if we're using the run command
    if args.len() > 1 && args[1] == "run" {
        // Check for -h/--help to show pytron's help
        if args.len() > 2 && (args[2] == "-h" || args[2] == "--help") {
            // Use clap's parser to show help - this will display help and exit
            Cli::parse_from(vec!["pytron", "run", "--help"]);
            return; // This line won't be reached as clap will exit after showing help
        }

        // Default values
        let mut zipfile = "robot.zip".to_string();
        let script = "main.py".to_string();
        let mut script_args = Vec::new();
        let mut uv_args = Vec::new();

        // Parse custom args using double-dash as separator
        let mut i = 2;
        let mut found_separator = false;
        let mut found_script_name = false;
        // Password for decrypting the ZIP file
        let mut password = None;
        let mut verification_path = None;

        while i < args.len() {
            if args[i] == "--signed" {
                // next element must be the password
                if i + 1 < args.len() {
                    verification_path = Some(args[i + 1].clone());
                    if let Some(path) = &verification_path {
                        println!("Key path found from args: {}", path);
                    }
                    i += 2;
                } else {
                    verification_path = std::env::var("PYTRON_SIGNATURE_KEY").ok();
                    if let Some(path) = &verification_path {
                        println!("Key path found from environment variable: {}", path);
                    } else {
                        verification_path = Some(zipfile.replace(".zip", ".key"));
                        
                        if let Some(path) = &verification_path {
                            println!("Key path defaults to: {}", path);
                        } else {
                            eprintln!("Error: Key not found.");
                            std::process::exit(1);
                        }
                    }
                    i += 1;
                }
                continue;
            }

            if args[i] == "--password" || args[i] == "-p" {
                // next element must be the password
                if i + 1 < args.len() {
                    password = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                } else {
                    eprintln!("Error: `{}` requires a value", args[i]);
                    std::process::exit(1);
                }
            }
            // Check for the double-dash separator
            if args[i] == "--" && !found_separator {
                found_separator = true;
                i += 1;
                continue;
            }

            // After separator or script name, everything goes to script
            if found_separator || found_script_name {
                script_args.push(args[i].clone());
                i += 1;
                continue;
            }

            // If not a flag and no script name found yet, treat as script/zipfile name
            if !args[i].starts_with('-') {
                zipfile = args[i].clone();
                found_script_name = true;
                i += 1;
                continue;
            }

            // Handle special case for --uv-run-help/-hh flags
            if args[i] == "--uv-run-help" || args[i] == "-hh" {
                // Convert to standard help flag for uv
                uv_args.push("--help".to_string());
                // Skip zip file execution completely when -hh is used
                println!("Running: uv run --help");
                // Check if uv is installed or download it if needed
                if !pytron::is_uv_installed() {
                    println!("uv not found. Attempting to download...");
                    match pytron::download_uv() {
                        Ok(path) => println!("Downloaded uv to: {}", path.display()),
                        Err(err) => {
                            eprintln!("Failed to download uv: {}. Please install uv manually (https://github.com/astral-sh/uv)", err);
                            exit(1);
                        }
                    }
                }
                // Run uv directly with help flag
                let status = pytron::get_uv_command().args(["run", "--help"]).status();
                match status {
                    Ok(status) => exit(status.code().unwrap_or(1)),
                    Err(err) => {
                        eprintln!("Error running uv: {}", err);
                        exit(1);
                    }
                }
            }

            // Everything else before separator or script name is a uv flag
            uv_args.push(args[i].clone());
            i += 1;
        }

        println!("Running: {}", script);
        println!("UV args: {:?}", uv_args);
        println!("Script args: {:?}", script_args);

        // Check if the first arg is a zipfile or a direct script
        let path = std::path::Path::new(&zipfile);
        let exit_code = if path
            .extension().is_some_and(|ext| ext == "zip" || ext == "ZIP")
        {
            // It's a zipfile, run from zip
            println!("Running from zip: {}", zipfile);

            // Don't pass the script as an argument again, it will be handled by run_from_zip
            // If script is in script_args, remove it
            let filtered_script_args: Vec<String> = script_args
                .iter()
                .filter(|&arg| arg != &script)
                .cloned()
                .collect();

            // Pass uv_args and script_args separately
            match pytron::run_from_zip(&zipfile, password.as_ref(), verification_path.as_ref(), &script, &uv_args, &filtered_script_args) {
                Ok(code) => code,
                Err(err) => {
                    eprintln!("Error running from zip: {}", err);
                    1
                }
            }
        } else {
            // It's a script, run directly
            println!("Running script directly: {}", zipfile);

            // Check if uv is installed or download it if needed
            if !pytron::is_uv_installed() {
                println!("uv not found. Attempting to download...");
                match pytron::download_uv() {
                    Ok(path) => println!("Downloaded uv to: {}", path.display()),
                    Err(err) => {
                        eprintln!("Failed to download uv: {}. Please install uv manually (https://github.com/astral-sh/uv)", err);
                        exit(1);
                    }
                }
            }

            // In this case, zipfile is actually the script path
            let mut cmd_args = vec!["run".to_string()];

            // Add uv args
            cmd_args.extend_from_slice(&uv_args);

            // Add script path
            cmd_args.push(zipfile.clone());

            // Add script args
            cmd_args.extend_from_slice(&script_args);

            println!("Running: uv {}", cmd_args.join(" "));

            // Run the script using uv with our helper function
            match pytron::get_uv_command().args(&cmd_args).status() {
                Ok(status) => status.code().unwrap_or(1),
                Err(err) => {
                    eprintln!("Error running script: {}", err);
                    1
                }
            }
        };

        exit(exit_code);
    } else {
        // Use clap for all other commands
        let cli = Cli::parse();

        match &cli.command {
            Commands::Zip {
                directory,
                output,
                ignore_patterns,
                password,
                sign,
            } => {
                if let Err(err) =
                    pytron::zip_directory(directory, output, ignore_patterns.as_ref(), password.as_ref(), sign)
                {
                    eprintln!("Error zipping directory: {}", err);
                    exit(1);
                }
            }
            Commands::Run {
                zipfile,
                password,
                signed,
                script,
                uv_args,
                script_args,
            } => {
                // Check if uv is installed or download it if needed
                if !pytron::is_uv_installed() {
                    println!("uv not found. Attempting to download...");
                    match pytron::download_uv() {
                        Ok(path) => println!("Downloaded uv to: {}", path.display()),
                        Err(err) => {
                            eprintln!("Failed to download uv: {}. Please install uv manually (https://github.com/astral-sh/uv)", err);
                            exit(1);
                        }
                    }
                }
                
                // This branch is for when using clap with -- to pass args
                let exit_code = match pytron::run_from_zip(zipfile, password.as_ref(), signed.as_ref(), script, uv_args, script_args) {
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
}
