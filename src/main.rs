extern crate clap;
extern crate colored;
extern crate regex;

use clap::{App, Arg, SubCommand};
use std::process::Command;
use colored::*;
use regex::Regex;

struct SystemResult {
    stdout: String,
    stderr: String,
    status: i32
}

impl SystemResult {
    fn new(output: std::process::Output) -> SystemResult {
        let mut stdout: Vec<char> = std::str::from_utf8(&output.stdout[..]).unwrap().to_string().chars().collect();
        stdout.pop();
        let stdout: String = stdout.into_iter().collect();
        let mut stderr: Vec<char> = std::str::from_utf8(&output.stderr[..]).unwrap().to_string().chars().collect();
        stderr.pop();
        let stderr: String = stderr.into_iter().collect();
        let mut result = SystemResult {
            stdout: stdout,
            stderr: std::str::from_utf8(&output.stderr[..]).unwrap().to_string(),
            status: 0
        };
        if result.stderr.chars().count() > 0 {
            result.status = 1
        }
        result
    }
}

fn system(command: &str) -> SystemResult {
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    let result = SystemResult::new(result);
    if result.status != 0 {
        let emsg = [
            "== ".red().to_string(),
            "[+]ERROR".red().bold().to_string(),
            " =====================".red().to_string()
        ]
        .join("");
        println!("{}", emsg);
        print!("{}", result.stderr);
        println!("{}", "=================================".red().to_string());
    }
    result
}

fn regex(re_str: &str) -> Regex {
    Regex::new(re_str).unwrap()
}

fn status() -> String {
    let result = system("git status --short").stdout;
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
    system(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    println!("{}", status());
}

fn commit() -> String {
    system("git commit").stdout
}

fn commit_with_message(message: &str) -> String {
    system(["git commit -m", message].join(" ").as_str()).stdout
}

fn commit_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("commit").unwrap().is_present("message") {
        commit_with_message(matches.subcommand_matches("commit").unwrap().value_of("message").unwrap());
        println!("{}", system("git log --decorate=short --oneline -1 --color").stdout);
    } else {
        commit();
        println!("{}", system("git log --decorate=short --oneline -1 --color").stdout);
    }
}

fn log(num: i32) -> String {
    system(["git log --decorate=short --oneline --color -", num.to_string().as_str()].join("").as_str()).stdout
}

fn log_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("log").unwrap().is_present("num") {
        println!("{}", log(matches.subcommand_matches("log").unwrap().value_of("num").unwrap().parse().unwrap()));
    } else {
        println!("{}", log(3));
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
