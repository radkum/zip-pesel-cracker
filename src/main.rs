mod cli;

use chrono::{Datelike, NaiveDate};
use std::fs::File;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use zip::{ZipArchive, read::ZipFile};

fn parse_file(file: &mut ZipFile, i: usize) -> Result<(), Box<dyn std::error::Error>> {
    let outpath = match file.enclosed_name() {
        Some(path) => path,
        None => return Ok(()),
    };

    {
        let comment = file.comment();
        if !comment.is_empty() {
            println!("File {i} comment: {comment}");
        }
    }

    if file.is_dir() {
        println!("File {} extracted to \"{}\"", i, outpath.display());
        std::fs::create_dir_all(&outpath).unwrap();
    } else {
        println!(
            "File {} extracted to \"{}\" ({} bytes)",
            i,
            outpath.display(),
            file.size()
        );
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                std::fs::create_dir_all(p).unwrap();
            }
        }
        let mut outfile = std::fs::File::create(&outpath).unwrap();
        io::copy(file, &mut outfile).unwrap();
    }
    Ok(())
}

fn decrypt_zip_file(
    archive: &mut ZipArchive<File>,
    file_index: usize,
    prefix: &str,
    combinations_digits_num: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let base: usize = 10;
    let combinations = base.pow(combinations_digits_num as u32 + 1) - 1;
    for i in 0..combinations {
        let unwanted_prefix = 6 - combinations_digits_num;
        let pass = format!("{:06}", i);
        let pass = pass[unwanted_prefix..].to_string();

        println!("{}", pass.as_str());
        if let Ok(mut file) =
            archive.by_index_decrypt(file_index, format!("{prefix}{pass}").as_bytes())
        {
            println!("Success! Password: {}", pass.as_str());
            return parse_file(&mut file, i);
        }
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    let cli = cli::ConfigCli::parse();
    if let Some(input_path) = cli.input_file {
        let mut prefix = String::new();
        let mut comb = 0;

        if let Some(commands) = cli.commands {
            match commands {
                cli::Commands::Birth(cli::StrDate { date }) => {
                    match NaiveDate::from_str(date.as_str()) {
                        Ok(date) => {
                            let year = date.year() % 100;
                            if year >= 2000 {
                                prefix = format!(
                                    "{:02}{:02}{:02}",
                                    year,
                                    date.month0() + 1,
                                    date.day0() + 1
                                );
                            } else {
                                prefix = format!(
                                    "{:02}{:02}{:02}",
                                    year,
                                    date.month0() + 21,
                                    date.day0() + 1
                                );
                            }
                        }
                        Err(err) => panic!("Failed to parse birth date: {err:?}"),
                    }
                }
                cli::Commands::BruteForce(cli::Brute { num }) => {
                    if num > 6 {
                        panic!(
                            "Incorrect number of combinations. Combination need to be up to 6 digits"
                        )
                    }
                    comb = num
                }
            }
        }

        return crack(input_path.as_path(), prefix.as_str(), comb);
    }

    println!("Hi! You can crack your pesel password now!\n");
    println!("First you need to print path to file to crack");

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let input_file = PathBuf::from(buffer);
    if !input_file.exists() {
        println!("Please, check if file exists");
        return Err(Box::new(std::io::Error::new(
            ErrorKind::NotFound,
            "Can't find input path",
        )));
    }

    println!("\nYou have two options:");
    println!("A) give your birth date and then we try to crack a whole pesel password");
    println!("B) give a count of last digits to check");
    println!("Type what option you choose");
    // `read_line` returns `Result` of bytes read
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        match buffer.trim_end() {
            "A" | "a" => {
                println!("Type your birth date");
                std::io::stdin().read_line(&mut buffer)?;
                return crack(input_file.as_path(), buffer.as_str(), 0);
            }
            "B" | "b" => {
                println!("Type your birth date");
                std::io::stdin().read_line(&mut buffer)?;
                let comb =
                    u32::from_str(buffer.as_str()).expect("You need input number, not a string");
                return crack(input_file.as_path(), "", comb as usize);
            }
            "Q" | "q" => {
                println!("Bye!");
                return Ok(());
            }
            _ => println!("Ah, you need to choose A or B. Q to quit"),
        }
    }
}

fn crack(
    input_path: &Path,
    prefix: &str,
    combinations_digits_num: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = std::fs::File::open(input_path).unwrap();
    let mut archive = zip::ZipArchive::new(file_name)?;
    for i in 0..archive.len() {
        decrypt_zip_file(&mut archive, i, prefix, combinations_digits_num)?;
    }
    Ok(())
}
