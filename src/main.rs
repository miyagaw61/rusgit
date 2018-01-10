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
    let mut result: Vec<char> = result.chars().collect();
    result.pop();
    let result: String = result.into_iter().collect();
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

fn status_trigger(){
    println!("{}", status());
}

fn add(files: Vec<&str>) {
    shell(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    println!("{}", status());
}

fn commit() -> String {
    shell("git commit")
}

fn commit_with_message(message: &str) -> String {
    shell(["git commit -m", message].join(" ").as_str())
}

fn commit_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("commit").unwrap().is_present("message") {
        commit_with_message(matches.subcommand_matches("commit").unwrap().value_of("message").unwrap());
        println!("{}", shell("git log --decorate=short --oneline -1 --color"));
    } else {
        commit();
        println!("{}", shell("git log --decorate=short --oneline -1 --color"));
    }
}

fn log(num: i32) -> String {
    shell(["git log --decorate=short --oneline --color -", num.to_string().as_str()].join("").as_str())
}

fn log_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("log").unwrap().is_present("num") {
        print!("{}", log(matches.subcommand_matches("log").unwrap().value_of("num").unwrap().parse().unwrap()));
    } else {
        print!("{}", log(3));
    }
}

fn main() {
    let matches = App::new("rusgit")
        .version("0.1.0")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("Git Wrapper in Rust")
        .subcommand(SubCommand::with_name("status"))
        .subcommand(SubCommand::with_name("add")
                    .arg(Arg::with_name("files")
                         .help("victim files")
                         .required(true)
                         .multiple(true)
                         )
                    )
        .subcommand(SubCommand::with_name("commit")
                    .arg(Arg::with_name("message")
                         .help("commit message")
                         )
                    )
        .subcommand(SubCommand::with_name("log")
                    .arg(Arg::with_name("num")
                         .help("num of logs")
                         )
                    )
        .get_matches();

    let sub_command = matches.subcommand_name().unwrap_or("");
    match sub_command {
        "status" => status_trigger(),
        "add" => add_trigger(&matches),
        "commit" => commit_trigger(&matches),
        "log" => log_trigger(&matches),
        _ => println!("something else.")
    } 
}
