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
        println!("{}", result.stderr);
        println!("{}", "=================================".red().to_string());
    }
    result
}

fn system_allow_stderr(command: &str) -> SystemResult {
    let result = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    SystemResult::new(result)
}

fn process(command: &str) -> std::process::ExitStatus {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("failed to execute process");
    child.wait().unwrap()
}

//fn regex(re: &str) -> Regex {
//    Regex::new(re).unwrap()
//}

fn status(mode: &str) {
    process("ls --color=always");
    let status = system_allow_stderr("git status --short").stdout;
    if status.chars().count() == 0 { std::process::exit(0); }
    println!("{}", "\n[+]GIT_STATUS".red().bold().to_string());
    println!("{}", "=============".yellow().bold().to_string());
    if mode == "verbose" {
        process("git status");
    } else {
        process("git status --short 2> /dev/null");
    }
}

fn status_trigger(matches: &clap::ArgMatches){
    if matches.subcommand_matches("status").unwrap().is_present("verbose") {
        status("verbose");
    } else {
        status("");
    }
}

fn add(files: Vec<&str>) {
    process(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    status("");
}

fn commit(message: &str) {
    if message == "" {
        process("git commit 1> /dev/null");
    } else {
        process(["git commit -m", message, "1> /dev/null"].join(" ").as_str());
    }
}

fn commit_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("commit").unwrap().is_present("amend") {
        process("git commit --amend");
    } else if matches.subcommand_matches("commit").unwrap().is_present("message") {
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
          "git diff",
          file
    ].join(" ").as_str());
}

fn diff_hash(hash: &str) {
    let cmd = ["git diff ", hash, "^..", hash].join("");
    process(cmd.as_str());
}

fn diff_cached(file: &str) {
    process([
            "git diff",
            file,
            "--cached"
    ].join(" ").as_str());
}

fn diff_trigger(matches: &clap::ArgMatches) {
    let re: Regex = Regex::new(r"[0123456789abcdef]{7}").unwrap();
    let file = matches.subcommand_matches("diff").unwrap().value_of("file | ..<branch> | <branch>..").unwrap();
    if file.chars().count() == 7 {
        if re.is_match(file) {
            diff_hash(file);
            std::process::exit(0);
        }
    }
    if matches.subcommand_matches("diff").unwrap().is_present("cached") {
        diff_cached(file);
        std::process::exit(0);
    }
    let file_vec: Vec<char> = file.chars().collect();
    if file.contains("..") {
        let chars_count = file.chars().count();
        if file_vec[0] == '.' && file_vec[1] == '.' {
            diff([
                 "HEAD..origin/".to_string(),
                 file_vec[2..].iter().collect()
            ].join("").as_str());
        } else if file_vec[chars_count-1] == '.' && file_vec[chars_count-2] == '.' {
            diff([
                 "origin/".to_string(),
                 file_vec[..chars_count-2].iter().collect(),
                 "..HEAD".to_string()
            ].join("").as_str());
        }
        std::process::exit(0);
    }
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

fn push(branch_name: &str) {
    if branch_name == "" {
        let now = branch("");
        process(["git push origin", now.as_str()].join(" ").as_str());
    } else {
        process(["git push origin", branch_name].join(" ").as_str());
    }
}

fn push_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("push").unwrap().is_present("branch") {
        let branch_name: &str = matches.subcommand_matches("push").unwrap().value_of("branch").unwrap();
        println!("{}", ["[+]PUSH: ", branch_name].join("").as_str().red().bold().to_string());
        print!("{}", "=========".yellow().bold().to_string());
        for _ in branch_name.chars() {
            print!("{}", "=".yellow().bold().to_string());
        }
        println!("");
        push(branch_name);
    } else {
        push("");
    }
}

fn get_branches() -> Vec<String> {
    let mut branches: Vec<String> = Vec::new();
    branches.push(branch(""));
    let iter = system("git branch").stdout;
    let iter = iter.split("\n");
    for x in iter {
        let x_chars: Vec<char> = x.chars().collect();
        let x_chars: &[char] = &x_chars;
        if x_chars[0] == '*' {
            continue;
        }
        branches.push(x[2..].to_string());
    }
    branches
}

fn branch(branch_name: &str) -> String {
    let before = system("git symbolic-ref --short HEAD 2> /dev/null").stdout;
    if branch_name == "" {
        return before
    }
    let branches = get_branches();
    if ! branches.contains(&branch_name.to_string()) {
        process(["git branch", branch_name].join(" ").as_str());
        println!("{}", ["Created branch", branch_name].join(" "));
        return "".to_string()
    }
    let before = before.as_str().red().bold().to_string();
    process(["git checkout", branch_name, "1> /dev/null 2> /dev/null"].join(" ").as_str());
    let arrow = " -> ".yellow().bold().to_string();
    println!("{}{}{}", before, arrow, branch_name.red().bold().to_string());
    "".to_string()
}

fn branch_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("branch").unwrap().is_present("branch") {
        let branch_name = matches.subcommand_matches("branch").unwrap().value_of("branch").unwrap();
        branch(branch_name);
    } else if matches.subcommand_matches("branch").unwrap().is_present("delete") {
        process(["git branch --delete", matches.subcommand_matches("branch").unwrap().value_of("delete").unwrap()].join(" ").as_str());
    } else if matches.subcommand_matches("branch").unwrap().is_present("DELETE") {
        process(["git branch -D", matches.subcommand_matches("branch").unwrap().value_of("DELETE").unwrap()].join(" ").as_str());
    } else if matches.subcommand_matches("branch").unwrap().is_present("remote-delete") {
        process(["git push --delete origin", matches.subcommand_matches("branch").unwrap().value_of("remote-delete").unwrap()].join(" ").as_str());
    } else {
        process("git branch");
    }
}

fn pull(branch_name: &str, mr: &str) {
    let branch_name: String = match branch_name {
        "" => branch(""),
        _ => branch_name.to_string()
    };
    process("git fetch");
    let cmd = match mr {
        "merge" => ["git merge origin/", &branch_name].join(""),
        "rebase" => ["git rebase origin/", &branch_name].join(""),
        _ => "".to_string()
    };
    process(&cmd);
}

fn pull_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("pull").unwrap().is_present("branch") {
        if matches.subcommand_matches("pull").unwrap().is_present("rebase") {
            pull(matches.subcommand_matches("pull").unwrap().value_of("branch").unwrap(), "rebase");
        } else {
            pull(matches.subcommand_matches("pull").unwrap().value_of("branch").unwrap(), "merge");
        }
    } else {
        if matches.subcommand_matches("pull").unwrap().is_present("rebase") {
            pull("", "rebase");
        } else {
            pull("", "merge");
        }
    }
}

fn merge_trigger(matches: &clap::ArgMatches) {
    let branch_name = matches.subcommand_matches("merge").unwrap().value_of("branch").unwrap();
    merge(branch_name);
}

fn merge(branch_name: &str) {
    process([
            "git merge",
            branch_name
    ].join(" ").as_str());
}

fn main() {
    let matches = App::new("rusgit")
        .version("0.1.0")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("Git Wrapper in Rust")
        .subcommand(SubCommand::with_name("status")
                    .arg(Arg::with_name("verbose")
                         .help("verbose status")
                         .short("v")
                         .long("verbose")
                         )
                    )
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
                    .arg(Arg::with_name("amend")
                         .help("git commit --amend")
                         .short("a")
                         .long("amend")
                         )
                    )
        .subcommand(SubCommand::with_name("log")
                    .arg(Arg::with_name("num")
                         .help("num of logs")
                         )
                    )
        .subcommand(SubCommand::with_name("diff")
                    .arg(Arg::with_name("file | ..<branch> | <branch>..")
                         .required(true)
                         )
                    .arg(Arg::with_name("cached")
                         .help("git diff --cached")
                         .short("c")
                         .long("cached")
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
                    .arg(Arg::with_name("delete")
                         .help("delete branch")
                         .short("d")
                         .long("delete")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("DELETE")
                         .help("force delete branch")
                         .short("D")
                         .long("DELETE")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("remote-delete")
                         .help("delete remote branch")
                         .long("remote-delete")
                         .takes_value(true)
                         )
                    )
        .subcommand(SubCommand::with_name("pull")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         )
                    .arg(Arg::with_name("rebase")
                         .help("git pull --rebase")
                         .short("r")
                         .long("rebase")
                         )
                    )
        .subcommand(SubCommand::with_name("merge")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         .required(true)
                         )
                    )
        .get_matches();

    let sub_command = matches.subcommand_name().unwrap_or("");

    match sub_command {
        "status" => status_trigger(&matches),
        "add" => add_trigger(&matches),
        "commit" => commit_trigger(&matches),
        "log" => log_trigger(&matches),
        "diff" => diff_trigger(&matches),
        "push" => push_trigger(&matches),
        "branch" => branch_trigger(&matches),
        "pull" => pull_trigger(&matches),
        "merge" => merge_trigger(&matches),
        "ac" => ac_trigger(&matches),
        _ => help()
    } 
}
