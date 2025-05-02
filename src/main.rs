use clap::{Parser, Subcommand};
use ignore::WalkBuilder;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{exit, Command};
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

    /// Extract and run a script from the zip archive
    Run {
        /// Path to the zip file
        #[arg(default_value = "robot.zip")]
        zipfile: String,

        /// Script to run from the zip
        #[arg(default_value = "main.py")]
        script: String,

        /// Arguments to pass to the script
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Zip { directory, output, ignore_patterns } => {
            if let Err(err) = zip_directory(directory, output, ignore_patterns.as_ref()) {
                eprintln!("Error zipping directory: {}", err);
                exit(1);
            }
        }
        Commands::Run {
            zipfile,
            script,
            args,
        } => {
            let exit_code = match run_from_zip(zipfile, script, args) {
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

pub fn zip_directory(directory: &str, output: &str, ignore_patterns: Option<&Vec<String>>) -> io::Result<()> {
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
        },
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
                let combined_patterns = default_ignores.iter()
                    .chain(patterns.iter())
                    .map(|s| s.to_string());
                
                explicit_ignores.extend(combined_patterns);
            }
            println!("Using ignore patterns: {:?}", explicit_ignores);
        },
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
                    
                    // Get path string for full path matching
                    let rel_path_str = rel_path.to_string_lossy();

                    if pattern.starts_with("*.") {
                        // Handle extension patterns like "*.log"
                        let ext = &pattern[1..]; // Get ".log"
                        file_name.ends_with(ext)
                    } else if pattern.ends_with("*") && pattern.starts_with("*") && pattern.len() > 2 {
                        // Handle middle patterns like "*custom_ignore*"
                        let middle = &pattern[1..pattern.len() - 1];
                        rel_path_str.contains(middle)
                    } else if pattern.ends_with("*") {
                        // Handle prefix patterns like "prefix*"
                        let prefix = &pattern[..pattern.len() - 1];
                        rel_path_str.starts_with(prefix)
                    } else if pattern.starts_with("*") {
                        // Handle suffix patterns like "*suffix"
                        let suffix = &pattern[1..];
                        rel_path_str.ends_with(suffix)
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

                    zip.start_file(rel_path.to_string_lossy(), options)?;

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

pub fn run_from_zip(zipfile: &str, script_path: &str, args: &[String]) -> io::Result<i32> {
    // Create a temporary directory for extraction
    let temp_dir = tempdir()?;

    println!("Extracting {} to temporary directory", zipfile);

    // Open the zip file
    let file = File::open(zipfile)?;
    let mut archive = ZipArchive::new(file)?;

    // Extract all files
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = temp_dir.path().join(file.name());

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

    // Prepare the command
    let mut cmd_args = vec![
        "run".to_string(),
        script_full_path.to_string_lossy().to_string(),
    ];
    cmd_args.extend_from_slice(args);

    println!("Running: uv {}", cmd_args.join(" "));

    // Run the script using uv
    let status = Command::new("uv").args(&cmd_args).status()?;

    Ok(status.code().unwrap_or(1))
}
