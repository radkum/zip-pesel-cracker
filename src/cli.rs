use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Default)]
pub struct ConfigCli {
    #[arg(short = 'v', long = "verbose")]
    /// Turn on verbose logging
    pub verbose: bool,

    #[arg(short = 'i', long = "input_file")]
    /// Turn on verbose logging
    pub input_file: Option<PathBuf>,

    #[command(subcommand)]
    pub commands: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Performs only survey actions, saving data to disk and exiting
    Birth(StrDate),

    /// Try to brute force last digits of pesel. Max 6 digits
    BruteForce(Brute),
}

#[derive(Parser)]
pub struct StrDate {
    #[arg(long)]
    pub date: String,
}

#[derive(Parser)]
pub struct Brute {
    #[arg(long)]
    pub num: usize,
}
