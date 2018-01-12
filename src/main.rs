extern crate clap;
extern crate colored;
//extern crate regex;

use clap::{App, Arg, SubCommand};
use std::process::Command;
use colored::*;
//use regex::Regex;

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
            stderr: stderr,
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
        ].join("");
        println!("{}", emsg);
        print!("{}", result.stderr);
        println!("{}", "=================================".red().to_string());
    }
    result
}

fn process(command: &str) -> std::process::ExitStatus {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("failed to execute process");
    child.wait().unwrap()
}

//fn regex(re_str: &str) -> Regex {
//    Regex::new(re_str).unwrap()
//}

fn status() {
    process("ls --color=always");
    println!("{}", "\n[+]STATUS".red().to_string());
    println!("{}", "=========".red().bold().to_string());
    process("git status --short");
}

fn status_trigger(){
    status();
}

fn add(files: Vec<&str>) {
    process(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    status();
}

fn commit(message: &str) {
    if message == "" {
        process("git commit 1> /dev/null");
    } else {
        process(["git commit -m", message, "1> /dev/null"].join(" ").as_str());
    }
}

fn commit_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("commit").unwrap().is_present("message") {
        commit(matches.subcommand_matches("commit").unwrap().value_of("message").unwrap());
        process("git log --decorate=short --oneline -1 --color");
    } else {
        commit("");
        process("git log --decorate=short --oneline -1 --color");
    }
}

fn log(num: i32) {
    process(["git log --decorate=short --oneline --color -", num.to_string().as_str()].join("").as_str());
}

fn log_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("log").unwrap().is_present("num") {
        log(matches.subcommand_matches("log").unwrap().value_of("num").unwrap().parse().unwrap());
    } else {
        log(3);
    }
}

fn help() {
    process("/mnt/c/Users/miyagaw61/home/repos/rusgit/target/debug/rusgit -h");
}

fn diff(file: &str) {
    process([
          "git diff --color ",
          file
    ].join(" ").as_str());
}

fn diff_trigger(matches: &clap::ArgMatches) {
    let file = matches.subcommand_matches("diff").unwrap().value_of("file").unwrap();
    diff(file);
}

fn ac(files: Vec<&str>) {
    add(files);
    commit("");
}

fn ac_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("files").unwrap().collect();
    ac(files);
    process("git log --decorate=short --oneline -1 --color");
}

fn push(branch: &str) {
    if branch == "" {
        println!("now under develop.");
    } else {
        process(["git push origin", branch].join(" ").as_str());
    }
}

fn push_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("push").unwrap().is_present("branch") {
        let branch: &str = matches.subcommand_matches("push").unwrap().value_of("branch").unwrap();
        push(branch);
    } else {
        push("");
    }
}

fn branch(branch_name: &str) -> String {
    if branch_name == "" {
        let branches = system("git branch").stdout;
        let branches = branches.split("\n");
        for x in branches {
            let x_chars: Vec<char> = x.chars().collect();
            let x_chars: &[char] = &x_chars;
            if x_chars[0] != '*' {
                continue;
            }
            let now: &[char] = &x_chars[2..];
            let now: String = now.into_iter().collect();
            return now
        }
    } else {
        let before = branch("");
        let before = before.as_str().red().bold().to_string();
        process(["git checkout", branch_name, "2> /dev/null"].join(" ").as_str());
        let arrow = " --> ".yellow().bold().to_string();
        println!("{}{}{}", before, arrow, branch_name.red().bold().to_string());
    }
    "".to_string()
}

fn branch_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("branch").unwrap().is_present("branch") {
        let branch_name = matches.subcommand_matches("branch").unwrap().value_of("branch").unwrap();
        branch(branch_name);
    } else {
        process("git branch");
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
        .subcommand(SubCommand::with_name("diff")
                    .arg(Arg::with_name("file")
                         .help("file path")
                         .required(true)
                         )
                    )
        .subcommand(SubCommand::with_name("ac")
                    .arg(Arg::with_name("files")
                         .help("victim files")
                         .required(true)
                         .multiple(true)
                         )
                    )
        .subcommand(SubCommand::with_name("push")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         )
                    )
        .subcommand(SubCommand::with_name("branch")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         )
                    )
        .get_matches();

    let sub_command = matches.subcommand_name().unwrap_or("");

    match sub_command {
        "status" => status_trigger(),
        "add" => add_trigger(&matches),
        "commit" => commit_trigger(&matches),
        "log" => log_trigger(&matches),
        "diff" => diff_trigger(&matches),
        "push" => push_trigger(&matches),
        "branch" => branch_trigger(&matches),
        "ac" => ac_trigger(&matches),
        _ => help()
    } 
}
