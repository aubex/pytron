use pytron::{Cli, Commands};
use clap::Parser;

// Test CLI argument parsing
#[test]
fn test_cli_parsing() {
    // Test the Zip command with defaults
    let args = vec!["pytron", "zip"];
    let cli = Cli::parse_from(args);

    if let Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_none(), "Default ignore_patterns should be None");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Zip command with custom ignore patterns
    let args = vec!["pytron", "zip", "--ignore-patterns", "node_modules,*.log,*.tmp"];
    let cli = Cli::parse_from(args);

    if let Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_some(), "Custom ignore_patterns should be Some");
        let patterns = ignore_patterns.unwrap();
        assert_eq!(patterns.len(), 3, "Expected 3 ignore patterns");
        assert_eq!(patterns[0], "node_modules", "First pattern should be 'node_modules'");
        assert_eq!(patterns[1], "*.log", "Second pattern should be '*.log'");
        assert_eq!(patterns[2], "*.tmp", "Third pattern should be '*.tmp'");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Zip command with empty string to override defaults
    let args = vec!["pytron", "zip", "--ignore-patterns", ""];
    let cli = Cli::parse_from(args);

    if let Commands::Zip { directory, output, ignore_patterns } = cli.command {
        assert_eq!(directory, ".", "Default directory should be '.'");
        assert_eq!(
            output, "robot.zip",
            "Default output file should be 'robot.zip'"
        );
        assert!(ignore_patterns.is_some(), "Empty ignore_patterns should be Some");
        let patterns = ignore_patterns.unwrap();
        assert_eq!(patterns.len(), 1, "Expected 1 empty string");
        assert_eq!(patterns[0], "", "Pattern should be empty string");
    } else {
        panic!("Expected Zip command");
    }

    // Test the Run command with defaults
    let args = vec!["pytron", "run"];
    let cli = Cli::parse_from(args);

    if let Commands::Run {
        zipfile,
        script,
        args,
    } = cli.command
    {
        assert_eq!(
            zipfile, "robot.zip",
            "Default zip file should be 'robot.zip'"
        );
        assert_eq!(script, "main.py", "Default script should be 'main.py'");
        assert_eq!(args.len(), 0, "No script arguments expected");
    } else {
        panic!("Expected Run command");
    }

    // Test the Run command with custom values
    let args = vec!["pytron", "run", "custom.zip", "custom.py", "arg1", "arg2"];
    let cli = Cli::parse_from(args);

    if let Commands::Run {
        zipfile,
        script,
        args,
    } = cli.command
    {
        assert_eq!(zipfile, "custom.zip", "Custom zip file name not matched");
        assert_eq!(script, "custom.py", "Custom script name not matched");
        assert_eq!(args.len(), 2, "Expected 2 script arguments");
        assert_eq!(args[0], "arg1", "First argument should be 'arg1'");
        assert_eq!(args[1], "arg2", "Second argument should be 'arg2'");
    } else {
        panic!("Expected Run command");
    }
}