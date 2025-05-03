use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

// CLI types are already available for use in main.rs and tests
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Zip files in a directory into robot.zip respecting .gitignore
    Zip {
        /// Directory to zip
        #[arg(default_value = ".")]
        directory: String,

        /// Output zip filename
        #[arg(short, long, default_value = "robot.zip")]
        output: String,

        /// Additional patterns to ignore (besides .gitignore)
        /// These are treated as gitignore patterns
        /// Default patterns include ".git" directory
        /// Pass an empty string to override all default excludes
        #[arg(short, long, value_delimiter = ',')]
        ignore_patterns: Option<Vec<String>>,
    },

    #[command(
        override_usage = "pytron run [UV_ARGS] [ZIPFILE] [SCRIPT] [SCRIPT_ARGS]...",
        about = "Run a script - either directly or from a zip archive",
        long_about = "Run a script - either directly or from a zip archive\n\nArguments are separated using a double-dash (--) or by specifying a script/zipfile path:\n  - Arguments before -- or before the zipfile path are passed to uv run\n  - Arguments after -- or after the zipfile path are passed to the script\n\nSpecial flags:\n  -h/--help: Show this help message (pytron's help)\n  -hh/--uv-run-help: Show uv's help message"
    )]
    Run {
        #[arg(
            default_value = "robot.zip",
            help = "Path to the zip file or script",
            long_help = "Path to the zip file or script\nIf a zip file (.zip), will extract and run the specified script from it\nIf a Python file (.py), will run it directly using uv"
        )]
        zipfile: String,

        #[arg(
            default_value = "main.py",
            help = "Script to run from the zip (if zipfile is a zip archive)",
            long_help = "Script to run from the zip (if zipfile is a zip archive)\nIf running a script directly, this is optional"
        )]
        script: String,

        #[arg(
            value_name = "UV_ARGS",
            allow_hyphen_values = true,
            num_args = 0..,
            help = "Arguments passed to uv run (before the -- or zipfile)",
            long_help = "Arguments passed to the uv run command\nThese appear before the -- separator or before the zipfile path\nExamples:\n  --with-pip\n  --system-site-packages\n  -v (verbose)"
        )]
        uv_args: Vec<String>,

        #[arg(
            value_name = "SCRIPT_ARGS",
            allow_hyphen_values = true,
            last = true,
            num_args = 0..,
            help = "Arguments passed to your script (after the -- or zipfile)",
            long_help = "Arguments passed to your Python script\nThese appear after the -- separator or after the zipfile path\nExamples:\n  --verbose\n  --config=config.json\n  -o output.txt"
        )]
        script_args: Vec<String>,
    },
}

pub fn zip_directory(
    directory: &str,
    output: &str,
    ignore_patterns: Option<&Vec<String>>,
) -> io::Result<()> {
    let dir_path = Path::new(directory);
    let output_path = Path::new(output);

    // Create the zip file
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);

    // Walk the directory using ignore, which respects .gitignore
    let walker = WalkBuilder::new(dir_path)
        .hidden(false) // Process hidden files too, but respect .gitignore
        .git_ignore(true) // Use .gitignore rules
        .build();

    // Create .gitignore matcher
    let gitignore_path = dir_path.join(".gitignore");
    let mut explicit_ignores = Vec::new();

    // Check if user provided ignore patterns
    let default_ignores = vec![".git".to_string()];

    match ignore_patterns {
        // Empty string means override default excludes
        Some(patterns) if patterns.len() == 1 && patterns[0].is_empty() => {
            println!("Overriding default excludes (no default patterns will be used)");
            // Use only gitignore content, no default excludes
            if gitignore_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            explicit_ignores.push(line.to_string());
                        }
                    }
                }
            }
        }
        // User provided custom patterns, use those plus defaults
        Some(patterns) => {
            if gitignore_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
                    let lines = content.lines();
                    // Combine .gitignore content with default ignores and user-provided patterns
                    let combined_lines = lines
                        .chain(default_ignores.iter().map(|s| s.as_str()))
                        .chain(patterns.iter().map(|s| s.as_str()));

                    for line in combined_lines {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            explicit_ignores.push(line.to_string());
                        }
                    }
                }
            } else {
                // No .gitignore file, just use defaults and user patterns
                let combined_patterns = default_ignores
                    .iter()
                    .chain(patterns.iter())
                    .map(|s| s.to_string());

                explicit_ignores.extend(combined_patterns);
            }
            println!("Using ignore patterns: {:?}", explicit_ignores);
        }
        // No user patterns, use .gitignore plus default excludes
        None => {
            if gitignore_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&gitignore_path) {
                    let lines = content.lines();
                    let combined_lines = lines.chain(default_ignores.iter().map(|s| s.as_str()));
                    for line in combined_lines {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            explicit_ignores.push(line.to_string());
                        }
                    }
                }
            } else {
                // No .gitignore file, just use default excludes
                explicit_ignores.extend(default_ignores);
            }
        }
    }

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();

                // Skip the output zip file itself
                if path.canonicalize().ok() == output_path.canonicalize().ok() {
                    continue;
                }

                // Skip files that match explicit .gitignore patterns
                let rel_path = path
                    .strip_prefix(dir_path)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                let should_ignore = explicit_ignores.iter().any(|pattern| {
                    // Get filename for extension matching
                    let file_name = rel_path
                        .file_name()
                        .map(|f| f.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Get path string for full path matching and normalize to use forward slashes
                    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");

                    if pattern.starts_with("*.") {
                        // Handle extension patterns like "*.log"
                        let ext = &pattern[1..]; // Get ".log"
                        file_name.ends_with(ext)
                    } else if pattern.ends_with("*")
                        && pattern.starts_with("*")
                        && pattern.len() > 2
                    {
                        // Handle middle patterns like "*custom_ignore*"
                        let middle = &pattern[1..pattern.len() - 1];
                        rel_path_str.contains(middle)
                    } else if pattern.ends_with("*") {
                        // Handle prefix patterns like "prefix*"
                        let prefix = &pattern[..pattern.len() - 1];
                        rel_path_str.starts_with(prefix)
                    } else if let Some(stripped) = pattern.strip_prefix("*") {
                        // Handle suffix patterns like "*suffix"
                        rel_path_str.ends_with(stripped)
                    } else {
                        // Exact match
                        &*rel_path_str == pattern
                    }
                });

                if should_ignore {
                    println!("Ignoring: {}", rel_path.display());
                    continue;
                }

                if path.is_file() {
                    // Print progress
                    println!("Adding: {}", rel_path.display());

                    // Convert path to use forward slashes for cross-platform compatibility
                    let zip_path = rel_path.to_string_lossy().replace('\\', "/");
                    zip.start_file(&zip_path, options)?;

                    // Write file contents
                    let mut file = File::open(path)?;
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;
                }
            }
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }

    // Finalize the zip
    zip.finish()?;
    println!("Archive created successfully: {}", output);

    Ok(())
}

/// Checks if uv is installed and accessible in PATH
pub fn is_uv_installed() -> bool {
    Command::new("uv").arg("--version").output().is_ok()
}

/// Gets the pytron home directory path
pub fn get_pytron_home() -> PathBuf {
    // Check if PYTRON_HOME is set
    if let Ok(path) = env::var("PYTRON_HOME") {
        return PathBuf::from(path);
    }
    
    // Otherwise use a default location in the user's home directory
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join("pytron_home")
}

/// Get the path to the uv executable
pub fn get_uv_path() -> PathBuf {
    let pytron_home = get_pytron_home();
    
    // Check for uv directly in PYTRON_HOME first (this is where the install script puts it)
    let direct_path = if cfg!(windows) {
        pytron_home.join("uv.exe")
    } else {
        pytron_home.join("uv")
    };
    
    // If it exists directly in PYTRON_HOME, use that path
    if direct_path.exists() {
        return direct_path;
    }
    
    // Otherwise check in the bin subdirectory (older installations may use this location)
    if cfg!(windows) {
        pytron_home.join("bin").join("uv.exe")
    } else {
        pytron_home.join("bin").join("uv")
    }
}

/// Creates a command for uv, using installed path if available or custom path if not
pub fn get_uv_command() -> Command {
    if is_uv_installed() {
        Command::new("uv")
    } else {
        Command::new(get_uv_path())
    }
}

/// Install uv to the pytron home directory
pub fn install_uv() -> io::Result<()> {
    let install_path = get_pytron_home();
    
    // Create the directory if it doesn't exist
    fs::create_dir_all(&install_path)?;
    
    println!("Installing uv to {}", install_path.display());
    
    // Platform-specific installation
    #[cfg(windows)]
    {
        // Windows installation using PowerShell
        let ps_command = format!(
            "powershell -ExecutionPolicy ByPass -c {{$env:UV_UNMANAGED_INSTALL = \"{}\"; irm https://astral.sh/uv/install.ps1 | iex}}",
            install_path.display()
        );
        
        let status = Command::new("cmd")
            .args(&["/C", &ps_command])
            .status()?;
            
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to install uv on Windows"
            ));
        }
    }
    
    #[cfg(not(windows))]
    {
        // Unix installation using curl and sh
        let install_cmd = format!(
            "curl -LsSf https://astral.sh/uv/install.sh | env UV_UNMANAGED_INSTALL=\"{}\" sh",
            install_path.display()
        );
        
        let status = Command::new("sh")
            .args(["-c", &install_cmd])
            .status()?;
            
        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to install uv on Unix"
            ));
        }
    }
    
    // Show the expected path of the uv executable
    let uv_path = get_uv_path();
    println!("Successfully installed uv to {}", install_path.display());
    println!("uv command path: {}", uv_path.display());
    Ok(())
}

pub fn run_from_zip(
    zipfile: &str,
    script_path: &str,
    uv_args: &[String],
    script_args: &[String],
) -> io::Result<i32> {
    // Create a temporary directory for extraction inside PYTRON_HOME
    // Use our centralized get_pytron_home function for consistency
    let pytron_home = get_pytron_home();
    let temp_path = pytron_home.join("temp");
    
    // Create the temp directory if it doesn't exist
    fs::create_dir_all(&temp_path)?;
    
    // Create a unique directory using tempfile in our custom location
    let temp_dir = tempfile::Builder::new()
        .prefix("pytron_")
        .tempdir_in(temp_path)?;

    println!("Extracting {} to temporary directory: {}", zipfile, temp_dir.path().display());

    // Open the zip file
    let file = File::open(zipfile)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        // Normalize file path for cross-platform compatibility
        let normalized_name = file
            .name()
            .replace('/', std::path::MAIN_SEPARATOR_STR);
        let outpath = temp_dir.path().join(normalized_name);

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }

            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;

            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if file.name().ends_with(".py") || !file.name().contains('.') {
                    let metadata = outpath.metadata()?;
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&outpath, perms)?;
                }
            }
        }
    }

    // Construct the full path to the script
    let script_full_path = temp_dir.path().join(script_path);

    if !script_full_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Script {} not found in archive", script_path),
        ));
    }

    // Arguments are now passed separately, no need to separate them here

    // Prepare the command
    let mut cmd_args = vec!["run".to_string()];

    // Add uv flags/options
    cmd_args.extend_from_slice(uv_args);

    // Add script path
    cmd_args.push(script_full_path.to_string_lossy().to_string());

    // Add script arguments
    cmd_args.extend_from_slice(script_args);

    println!("Running: uv {}", cmd_args.join(" "));

    // Check if uv is installed and install if needed
    if !is_uv_installed() {
        println!("uv not found. Attempting to install...");
        match install_uv() {
            Ok(_) => println!("uv installed successfully."),
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to install uv: {}", e)
                ));
            }
        }
    }

    // Run the script using uv (using our helper function)
    let status = get_uv_command().args(&cmd_args).status()?;

    Ok(status.code().unwrap_or(1))
}
