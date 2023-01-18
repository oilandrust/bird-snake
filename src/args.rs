use clap::{Parser, Subcommand};

/// Cli API.
/// Run a level
/// ./snake-bird -l 0
/// ./snake-bird --level 0
/// Run a test level
/// ./snake-bird -t 0
/// ./snake-bird --test_level 0
/// // Run the automated tests
/// ./snake-bird test
/// // Run the automated tests for a specific test case
/// ./snake-bird -t 0 test

#[derive(Parser, Debug, Default)]
pub struct Args {
    #[arg(short, long)]
    pub level: Option<usize>,

    #[arg(short, long)]
    pub test_level: Option<usize>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run automated tests.
    Test {
        #[arg(short, long)]
        test_case: Option<usize>,
    },
}
