extern crate clap;
extern crate colored;
extern crate regex;

use clap::{App, Arg, SubCommand};
use std::process::Command;
use colored::*;
use regex::Regex;

fn shell(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    std::str::from_utf8(&output.stdout[..]).unwrap().to_string()
}

fn regex(re_str: &str) -> Regex {
    Regex::new(re_str).unwrap()
}

fn status() -> String {
    let result = shell("git status --short");
    let result = regex(r"\n$").replace(result.as_str(), "");
    let lines: Vec<&str> = result.split("\n").collect();
    let mut result_vec: Vec<String> = Vec::new();
    for x in lines {
        let x_array: Vec<char> = x.chars().collect();
        let mut r0: String = " ".to_string();
        let mut r1: String = " ".to_string();
        if x_array[0] == 'M' {
            r0 = "M".green().bold().to_string();
        }
        if x_array[1] == 'M' {
            r1 = "M".red().bold().to_string();
        }
        if x_array[0] == 'A' {
            r0 = "A".green().bold().to_string();
        }
        if x_array[0] == 'A' {
            r0 = "A".green().bold().to_string();
        }
        if x_array[0] == '?' && x_array[1] == '?'{
            r0 = "?".red().bold().to_string();
            r1 = "?".red().bold().to_string();
        }
        let x_string: String = x_array[2..].into_iter().collect();
        let result_line: String = [r0, r1, x_string].join("");
        result_vec.push(result_line);
    }
    result_vec.join("\n")
}

fn status_routine(){
    println!("{}", status());
}

fn add(files: Vec<&str>) {
    shell(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_routine(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    println!("{}", status());
}

fn main() {
    let matches = App::new("rusgit")
        .version("0.1.0")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("Git Wrapper in Rust")
        .subcommand(SubCommand::with_name("status")
                    .about("status subcommand")
                    )
        .subcommand(SubCommand::with_name("add")
                    .about("add subcommand")
                    .arg(Arg::with_name("files")
                         .help("victim files")
                         .required(true)
                         .multiple(true)
                         )
                    )
        .get_matches();

    let sub_command = matches.subcommand_name().unwrap_or("");
    match sub_command {
        "status" => status_routine(),
        "add" => add_routine(&matches),
        _ => println!("something else.")
    } 
}
