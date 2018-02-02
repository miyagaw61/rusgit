extern crate clap;
extern crate colored;
extern crate regex;

use std::fs::OpenOptions;
use std::io::Read;
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

fn status_with_ls(mode: &str, ls: &str) {
    process(&ls);
    let status = system_allow_stderr("git status --short").stdout;
    if status.chars().count() == 0 { std::process::exit(0); }
    println!("{}", "\n[+]GIT-STATUS".red().bold().to_string());
    println!("{}", "=============".yellow().bold().to_string());
    if mode == "verbose" {
        process("git status");
    } else {
        process("git status --short 2> /dev/null");
    }
}

fn status(mode: &str) {
    let status = system_allow_stderr("git status --short").stdout;
    if status.chars().count() == 0 { std::process::exit(0); }
    if mode == "verbose" {
        process("git status");
    } else {
        process("git status --short 2> /dev/null");
    }
}

fn status_trigger(matches: &clap::ArgMatches){
    if matches.subcommand_matches("status").unwrap().is_present("ls") {
        if matches.subcommand_matches("status").unwrap().is_present("verbose") {
            status_with_ls("verbose", matches.subcommand_matches("status").unwrap().value_of("ls").unwrap());
        } else {
            status_with_ls("", matches.subcommand_matches("status").unwrap().value_of("ls").unwrap());
        }
    } else {
        if matches.subcommand_matches("status").unwrap().is_present("verbose") {
            status("verbose");
        } else {
            status("");
        }
    }
}

fn add(files: Vec<&str>) {
    process(["git add", files.join(" ").as_str()].join(" ").as_str());
}

fn add_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("add").unwrap().is_present("p") {
        process("git add -p");
        std::process::exit(0);
    }
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    status("");
}

fn commit(message: &str) {
    let mut message_string = String::new();
    match message {
        "" => {
            let editor = std::env::var("RUSGIT_EDITOR").unwrap_or("".to_string());
            let editor = match &editor[..] {
                "" => std::env::var("EDITOR").unwrap_or("".to_string()),
                n => n.to_string()
            };
            process("rm -rf /tmp/rusgit_commit.tmp");
            if editor != "" {
                let cmd = [&editor,
                        " /tmp/rusgit_commit.tmp"
                ].join("");
                process(&cmd);
            } else {
                process("vim /tmp/rusgit_commit.tmp");
            }
            let mut f = match OpenOptions::new().read(true).write(true).open("/tmp/rusgit_commit.tmp") {
                Ok(f) => f,
                Err(_) => {
                    std::fs::File::create("/tmp/rusgit_commit.tmp").unwrap();
                    OpenOptions::new().read(true).write(true).open("/tmp/rusgit_commit.tmp").unwrap()
                }
            };
            f.read_to_string(&mut message_string).expect("can not read file");
        },
        _ => message_string = message.to_string()
    };
    let mut prefix: String = "".to_string();
    if message_string.chars().count() > 2 {
        prefix = match &message_string[0..2] {
            "i " => "Improve ".to_string(),
            "I " => "Implement ".to_string(),
            "r " => "Remove ".to_string(),
            "R " => "Refactor ".to_string(),
            "u " => "Use ".to_string(),
            "U " => "Update ".to_string(),
            "a " => "Add ".to_string(),
            "c " => "Change ".to_string(),
            "f " => "Fix ".to_string(),
            "s " => "Support ".to_string(),
            "l " => "Allow ".to_string(),
            "v " => "Avoid ".to_string(),
            _ => "".to_string()
        };
    }
    match &prefix[..] {
        "" => process(["git commit -m \"", &message_string, "\" 1> /dev/null"].join("").as_str()),
        n => process(["git commit -m \"", n, &message_string[2..], "\" 1> /dev/null"].join("").as_str())
    };
}

fn commit_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("commit").unwrap().is_present("amend") {
        process("git commit --amend");
    } else if matches.subcommand_matches("commit").unwrap().is_present("message") {
        let message: Vec<&str> = matches.subcommand_matches("commit").unwrap().values_of("message").unwrap().collect();
        let message: String = message.join(" ");
        commit(&message);
        process("git log --decorate=short --oneline -1 --color");
    } else {
        commit("");
        process("git log --decorate=short --oneline -1 --color");
    }
}

fn log(num: i32, options: Vec<&str>) {
    let mut cmd = "git log --decorate=short".to_string();
    for x in options {
        cmd.push(' ');
        cmd.push_str(x);
    }
    cmd.push_str(" -");
    match num {
        -1 => process([&cmd, "-all"].join("").as_str()),
        _ => process([&cmd, num.to_string().as_str()].join("").as_str())
    };
}

fn log_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("log").unwrap().is_present("ref") {
        match matches.subcommand_matches("log").unwrap().value_of("0").unwrap_or("") {
            "" => {
                reflog(branch("").as_str(), matches.subcommand_matches("log").unwrap().is_present("all"));
                std::process::exit(0);
            },
            branch_name => {
                reflog(branch_name, matches.subcommand_matches("log").unwrap().is_present("all"));
                std::process::exit(0);
            }
        }
    }
    let mut options: Vec<&str> = Vec::new();
    match matches.subcommand_matches("log").unwrap().is_present("graph") {
        true => options.push("--graph"),
        false => print!("")
    }
    match matches.subcommand_matches("log").unwrap().is_present("verbose") {
        true => options.push("-p"),
        false => options.push("--oneline")
    }
    if matches.subcommand_matches("log").unwrap().is_present("0") {
        match matches.subcommand_matches("log").unwrap().is_present("all") {
            true => log(-1, options),
            false => log(matches.subcommand_matches("log").unwrap().value_of("0").unwrap().parse().unwrap(), options)
        }
    } else {
        match matches.subcommand_matches("log").unwrap().is_present("all") {
            true => log(-1, options),
            false => log(3, options)
        }
    }
}

fn help() {
    println!("\
USAGE:
    rusgit [SUBCOMMAND]
rusgit -h for help\
");
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
            "git diff ",
            "--cached ",
            file,
    ].join("").as_str());
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

fn ac(files: Vec<&str>, message: &str) {
    add(files);
    commit(message);
}

fn ac_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("files").unwrap().collect();
    let mut message: String = if matches.subcommand_matches("ac").unwrap().is_present("message") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("message").unwrap().collect();
        let mut message: String = message.join(" ");
        message
    } else if matches.subcommand_matches("ac").unwrap().is_present("improve") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("improve").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Improve ", &message].join(" ")
    } else if matches.subcommand_matches("ac").unwrap().is_present("implement") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("implement").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Implement ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("refactor") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("refactor").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Refactor ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("use") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("use").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Use ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("update") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("update").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Update ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("add") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("add").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Add ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("change") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("change").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Change ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("fix") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("fix").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Fix ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("support") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("support").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Support ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("allow") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("allow").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Allow ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("avoid") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("avoid").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Avoid ", &message].join("")
    } else if matches.subcommand_matches("ac").unwrap().is_present("remove") {
        let mut message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("remove").unwrap().collect();
        let mut message: String = message.join(" ");
        ["Remove ", &message].join("")
    } else {
        "".to_string()
    };
    let last = message.chars().count()-1;
    if last > 1 {
        let last3: String = message.chars().skip(last-2).collect();
        if files.len() == 1 {
            match &last3[..] {
                " to" => message = [&message, files[0]].join(" "),
                " in" => message = [&message, files[0]].join(" "),
                _ => print!("")
            };
        }
    }
    if last > 2 {
        let last4: String = message.chars().skip(last-3).collect();
        if files.len() == 1 {
            match &last4[..] {
                " for" => message = [&message, files[0]].join(" "),
                _ => print!("")
            };
        }
    }
    ac(files, &message);
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
    let branch_name: String = if matches.subcommand_matches("push").unwrap().is_present("branch") {
        matches.subcommand_matches("push").unwrap().value_of("branch").unwrap().to_string()
    } else {
        branch("")
    };
    println!("{}", ["[+]PUSH: ", branch_name.as_str()].join("").as_str().red().bold().to_string());
    print!("{}", "=========".yellow().bold().to_string());
    for _ in branch_name.chars() {
        print!("{}", "=".yellow().bold().to_string());
    }
    println!("");
    push(&branch_name);
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
    let before = before.as_str().red().bold().to_string();
    let branches = get_branches();
    let result = if ! branches.contains(&branch_name.to_string()) {
        system_allow_stderr(["git checkout -b", branch_name, "1> /dev/null"].join(" ").as_str())
    } else {
        system_allow_stderr(["git checkout", branch_name, "1> /dev/null"].join(" ").as_str())
    };
    if result.stderr != "" {
        let stderr: Vec<&str> = result.stderr.split("\n").collect();
        let stderr_count = stderr.iter().count();
        if !(stderr_count == 1 && stderr[0].contains("Switched")) {
            println!("{}", "[+]ERROR:".red().bold().to_string());
            println!("{}", "=========".yellow().bold().to_string());
            println!("{}", result.stderr.red().bold().to_string());
            std::process::exit(0);
        }
    }
    if ! branches.contains(&branch_name.to_string()) {
        println!("{}", ["Created branch:", branch_name].join(" "));
    }
    let arrow = " -> ".yellow().bold().to_string();
    println!("{}{}{}", before, arrow, branch_name.red().bold().to_string());
    "".to_string()
}

fn branch_pull(branch_name: &str, remote_branch_name: &str) {
    let before = system("git symbolic-ref --short HEAD 2> /dev/null").stdout;
    let before = before.as_str().red().bold().to_string();
    let branches = get_branches();
    let result = if ! branches.contains(&branch_name.to_string()) {
        system_allow_stderr(["git checkout -b ", branch_name, " origin/", remote_branch_name, " 1> /dev/null"].join("").as_str())
    } else {
        println!("Already exists: {}", branch_name);
        std::process::exit(0);
    };
    if result.stderr != "" {
        let stderr: Vec<&str> = result.stderr.split("\n").collect();
        let stderr_count = stderr.iter().count();
        if !(stderr_count == 1 && stderr[0].contains("Switched")) {
            println!("{}", "[+]ERROR:".red().bold().to_string());
            println!("{}", "=========".yellow().bold().to_string());
            println!("{}", result.stderr.red().bold().to_string());
            std::process::exit(0);
        }
    }
    if ! branches.contains(&branch_name.to_string()) {
        println!("{}", ["Created branch:", branch_name].join(" "));
    }
    let arrow = " -> ".yellow().bold().to_string();
    println!("{}{}{}", before, arrow, branch_name.red().bold().to_string());
}

fn branch_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("branch").unwrap().is_present("remote-branch") {
        let branch_name = matches.subcommand_matches("branch").unwrap().value_of("branch").unwrap();
        let remote_branch_name = matches.subcommand_matches("branch").unwrap().value_of("remote-branch").unwrap();
        branch_pull(branch_name, remote_branch_name);
    } else if matches.subcommand_matches("branch").unwrap().is_present("branch") {
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
    let branch_name: String = if matches.subcommand_matches("pull").unwrap().is_present("branch") {
        matches.subcommand_matches("pull").unwrap().value_of("branch").unwrap().to_string()
    } else {
        branch("")
    };
    if matches.subcommand_matches("pull").unwrap().is_present("rebase") {
        println!("{}", ["[+]REBASE-PULL: ", branch_name.as_str()].join("").as_str().red().bold().to_string());
        print!("{}",    "================".yellow().bold().to_string());
        for _ in branch_name.chars() {
            print!("{}", "=".yellow().bold().to_string());
        }
        println!("");
    } else {
        println!("{}", ["[+]PULL: ", branch_name.as_str()].join("").as_str().red().bold().to_string());
        print!("{}",    "=========".yellow().bold().to_string());
        for _ in branch_name.chars() {
            print!("{}", "=".yellow().bold().to_string());
        }
        println!("");
    }
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

fn rebase_trigger(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("rebase").unwrap().is_present("i") {
        process([
                "git rebase -i ",
                matches.subcommand_matches("rebase").unwrap().value_of("i").unwrap()
        ].join("").as_str());
        std::process::exit(0);
    }
    let branch_name = matches.subcommand_matches("rebase").unwrap().value_of("branch").unwrap();
    rebase(branch_name);
}

fn rebase(branch_name: &str) {
    process([
            "git rebase",
            branch_name
    ].join(" ").as_str());
}

fn clone(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("clone").unwrap().is_present("directory") {
        process([
                "git clone ",
                "https://github.com/",
                matches.subcommand_matches("clone").unwrap().value_of("repo").unwrap(),
                " ",
                matches.subcommand_matches("clone").unwrap().value_of("directory").unwrap(),
                " "
        ].join("").as_str());
    } else {
        process([
                "git clone ",
                "https://github.com/",
                matches.subcommand_matches("clone").unwrap().value_of("repo").unwrap()
        ].join("").as_str());
    }
}

fn undo(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("undo").unwrap().is_present("orig") {
        if matches.subcommand_matches("undo").unwrap().subcommand_matches("orig").unwrap().is_present("hard") {
            process("git reset --hard ORIG_HEAD");
        } else {
            process("git reset --soft ORIG_HEAD");
        }
    } else if matches.subcommand_matches("undo").unwrap().is_present("commit") {
        if matches.subcommand_matches("undo").unwrap().subcommand_matches("commit").unwrap().is_present("commit id") {
            let commit_id: &str = matches.subcommand_matches("undo").unwrap().subcommand_matches("commit").unwrap().value_of("commit id").unwrap();
            let msg = [
                      "undo changes until ",
                      commit_id
            ].join("");
            let chars = msg.chars();
            let msg = msg.red().bold().to_string();
            println!("{}", msg);
            for _ in chars {
                print!("{}", "=".yellow().bold().to_string());
            }
            println!("");
            if matches.subcommand_matches("undo").unwrap().subcommand_matches("commit").unwrap().is_present("hard") {
                process([
                        "git reset --hard ",
                        commit_id
                ].join("").as_str());
            } else {
                process([
                        "git reset --soft ",
                        commit_id
                ].join("").as_str());
            }
        } else if matches.subcommand_matches("undo").unwrap().subcommand_matches("commit").unwrap().is_present("hard") {
            let msg = "undo commit(not keep changes)";
            let chars = msg.chars();
            let msg = msg.red().bold().to_string();
            println!("{}", msg);
            for _ in chars {
                print!("{}", "=".yellow().bold().to_string());
            }
            println!("");
            process("git reset --hard HEAD^");
        } else {
            let msg = "undo commit(keep changes)";
            let chars = msg.chars();
            let msg = msg.red().bold().to_string();
            println!("{}", msg);
            for _ in chars {
                print!("{}", "=".yellow().bold().to_string());
            }
            println!("");
            process("git reset --soft HEAD^");
        }
    } else if matches.subcommand_matches("undo").unwrap().is_present("add") {
        let file_name: &str = matches.subcommand_matches("undo").unwrap().subcommand_matches("add").unwrap().value_of("file").unwrap_or("");
        let msg = [
                  "undo stage: ",
                  file_name
        ].join("");
        let chars = msg.chars();
        let msg = msg.red().bold().to_string();
        println!("{}", msg);
        for _ in chars {
            print!("{}", "=".yellow().bold().to_string());
        }
        println!("");
        process([
                "git reset HEAD ",
                file_name
        ].join("").as_str());
    } else if matches.subcommand_matches("undo").unwrap().is_present("head") {
        let file_name: &str = matches.subcommand_matches("undo").unwrap().subcommand_matches("head").unwrap().value_of("file").unwrap();
        let msg = [
                  "undo changes: ",
                  file_name
        ].join("");
        let chars = msg.chars();
        let msg = msg.red().bold().to_string();
        println!("{}", msg);
        for _ in chars {
            print!("{}", "=".yellow().bold().to_string());
        }
        println!("");
        process(["git checkout HEAD ",
                file_name
        ].join("").as_str());
    }
}

fn tag_trigger(matches: &clap::ArgMatches) {
    let tag_name = matches.subcommand_matches("tag").unwrap().value_of("tag-name").unwrap_or("");
    if matches.subcommand_matches("tag").unwrap().is_present("show") {
        process([
                "git show ",
                tag_name
        ].join("").as_str());
    } else if tag_name == "" {
        process("git tag");
    } else if matches.subcommand_matches("tag").unwrap().is_present("editor") {
        process([
                "git tag -a ",
                tag_name
        ].join("").as_str());
        process([
                "git push origin ",
                tag_name
        ].join("").as_str());
    } else if matches.subcommand_matches("tag").unwrap().is_present("message") {
        let message: Vec<&str> = matches.subcommand_matches("tag").unwrap().values_of("message").unwrap().collect();
        let message: String = message.join(" ");
        process([
                "git tag -a ",
                tag_name,
                " -m \"",
                &message,
                "\""
        ].join("").as_str());
        process([
                "git push origin ",
                tag_name
        ].join("").as_str());
    } else if matches.subcommand_matches("tag").unwrap().is_present("delete") {
        process([
                "git tag -d ",
                tag_name
        ].join("").as_str());
        process([
                "git push --delete origin ",
                tag_name
        ].join("").as_str());
    } else {
        process([
                "git tag ",
                tag_name
        ].join("").as_str());
        process([
                "git push origin ",
                tag_name
        ].join("").as_str());
    }
}

fn reflog(branch_name: &str, all: bool) {
    if all {
        process("git reflog");
    } else if branch_name == "" {
        process([
                "git reflog ",
                branch("").as_str()
        ].join("").as_str());
    } else {
        process([
                "git reflog ",
                branch_name
        ].join("").as_str());
    }
}

fn complete(matches: &clap::ArgMatches) {
    let text = "\
rusgit_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    set ${COMP_WORDS[@]}
    if test \"$prev\" = \"rusgit\" ;then
        opts=\"ac add alias branch clone commit diff help log merge pull push rebase status tag undo\"
        COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
    elif test \"$(echo $2 | grep -E \"(ac|add|diff)\")\" ;then
        COMPREPLY=( $(compgen -f -- \"${cur}\") )
    elif test \"$(echo $2 | grep -E \"(branch|merge|pull|push|rebase)\")\" ;then
        opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
        COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
    elif test \"$2\" = \"undo\" ;then
        if test \"$(echo $3 | grep -E \"(add|head)\")\" ;then
            COMPREPLY=( $(compgen -f -- \"${cur}\") )
        else 
            opts=\"add commit head orig\"
            COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
        fi
    elif test \"$2\" = \"tag\" ;then
        if test \"$(echo $3 | grep -E \"(-d|-s)\")\" ;then
            opts=\"$(git tag)\"
            COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
        fi
    fi
}
complete -F rusgit_complete rusgit";
    println!("{}", text);
    if matches.subcommand_matches("complete").unwrap().is_present("ac") {
        let text = "\
rusgit__ac_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    COMPREPLY=( $(compgen -f -- \"${cur}\") )
}
complete -F rusgit__ac_complete _ac
alias _ac=\"rusgit ac\"";
        let text = text.replace("_ac", matches.subcommand_matches("complete").unwrap().value_of("ac").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("add") {
        let text = "\
rusgit__add_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    COMPREPLY=( $(compgen -f -- \"${cur}\") )
}
complete -F rusgit__add_complete _add
alias _add=\"rusgit add\"";
        let text = text.replace("_add", matches.subcommand_matches("complete").unwrap().value_of("add").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("diff") {
        let text = "\
rusgit__diff_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    COMPREPLY=( $(compgen -f -- \"${cur}\") )
}
complete -F rusgit__diff_complete _diff
alias _diff=\"rusgit diff\"";
        let text = text.replace("_diff", matches.subcommand_matches("complete").unwrap().value_of("diff").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("branch") {
        let text = "\
rusgit__branch_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
    COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
}
complete -F rusgit__branch_complete _branch
alias _branch=\"rusgit branch\"";
        let text = text.replace("_branch", matches.subcommand_matches("complete").unwrap().value_of("branch").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("merge") {
        let text = "\
rusgit__merge_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
    COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
}
complete -F rusgit__merge_complete _merge
alias _merge=\"rusgit merge\"";
        let text = text.replace("_merge", matches.subcommand_matches("complete").unwrap().value_of("merge").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("pull") {
        let text = "\
rusgit__pull_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
    COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
}
complete -F rusgit__pull_complete _pull
alias _pull=\"rusgit pull\"";
        let text = text.replace("_pull", matches.subcommand_matches("complete").unwrap().value_of("pull").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("push") {
        let text = "\
rusgit__push_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
    COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
}
complete -F rusgit__push_complete _push
alias _push=\"rusgit push\"";
        let text = text.replace("_push", matches.subcommand_matches("complete").unwrap().value_of("push").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("rebase") {
        let text = "\
rusgit__rebase_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
    COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
}
complete -F rusgit__rebase_complete _rebase
alias _rebase=\"rusgit rebase\"";
        let text = text.replace("_rebase", matches.subcommand_matches("complete").unwrap().value_of("rebase").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("undo") {
        let text = "\
rusgit__undo_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    set ${COMP_WORDS[@]}
    if test \"$(echo $2 | grep -E \"(add|head)\")\" ;then
        COMPREPLY=( $(compgen -f) )
    else 
        opts=\"add commit head orig\"
        COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
    fi
}
complete -F rusgit__undo_complete _undo
alias _undo=\"rusgit undo\"";
        let text = text.replace("_undo", matches.subcommand_matches("complete").unwrap().value_of("undo").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("tag") {
        let text = "\
rusgit__tag_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    set ${COMP_WORDS[@]}
    if test \"$(echo $2 | grep -E \"(-d|-s)\")\" ;then
        opts=\"$(git tag)\"
        COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
    fi
}
complete -F rusgit__tag_complete _tag
alias _tag=\"rusgit tag\"";
        let text = text.replace("_tag", matches.subcommand_matches("complete").unwrap().value_of("tag").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("log") {
        let text = "\
rusgit__log_complete() {
    local cur prev cword opts
    _get_comp_words_by_ref -n : cur prev cword
    set ${COMP_WORDS[@]}
    if test \"$2\" = \"-r\" ;then
        opts=\"$(git branch | sed -E 's/\\* //g' | sed -E 's/  //g')\"
        COMPREPLY=( $(compgen -W \"${opts}\" -- \"${cur}\") )
    fi
}
complete -F rusgit__log_complete _log
alias _log=\"rusgit log\"";
        let text = text.replace("_log", matches.subcommand_matches("complete").unwrap().value_of("log").unwrap());
        println!("{}", text);
    }

    if matches.subcommand_matches("complete").unwrap().is_present("commit") {
        println!("{}", ["alias ", matches.subcommand_matches("complete").unwrap().value_of("commit").unwrap(), "=\"rusgit commit\""].join("").as_str());
    }

    if matches.subcommand_matches("complete").unwrap().is_present("clone") {
        println!("{}", ["alias ", matches.subcommand_matches("complete").unwrap().value_of("clone").unwrap(), "=\"rusgit clone\""].join("").as_str());
    }

    if matches.subcommand_matches("complete").unwrap().is_present("status") {
        println!("{}", ["alias ", matches.subcommand_matches("complete").unwrap().value_of("status").unwrap(), "=\"rusgit status\""].join("").as_str());
    }
}

fn main() {
    let matches = App::new("rusgit")
        .version("2.0.0")
        .author("miyagaw61 <miyagaw61@gmail.com>")
        .about("Git Wrapper in Rust")
        .subcommand(SubCommand::with_name("status")
                    .about("improved git-status")
                    .arg(Arg::with_name("verbose")
                         .help("verbose status")
                         .short("v")
                         .long("verbose")
                         )
                    .arg(Arg::with_name("ls")
                         .help("status with ls command")
                         .long("ls")
                         .takes_value(true)
                         )
                    )
        .subcommand(SubCommand::with_name("add")
                    .about("improved git-add")
                    .arg(Arg::with_name("files")
                         .help("victim files")
                         .required_unless("p")
                         .multiple(true)
                         )
                    .arg(Arg::with_name("p")
                         .help("git add -p")
                         .short("p")
                         )
                    )
        .subcommand(SubCommand::with_name("commit")
                    .about("improved git-commit")
                    .arg(Arg::with_name("message")
                         .help("commit message")
                         .multiple(true)
                         )
                    .arg(Arg::with_name("amend")
                         .help("git commit --amend")
                         .short("a")
                         .long("amend")
                         )
                    )
        .subcommand(SubCommand::with_name("log")
                    .about("improved git-log")
                    .help("\
rusgit-log
improved git-log

USAGE:
    rusgit log [FLAGS] [OPTIONS] [num]

FLAGS:
    -a, --all        show all
    -g, --graph      graph mode
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    verbose mode

OPTIONS:
    -r, --ref [branch]    reflog

ARGS:
    <num>    num of logs [default: 3]\
")
                    .arg(Arg::with_name("0")
                         .help("num of logs")
                         .value_name("num")
                         )
                    .arg(Arg::with_name("graph")
                         .help("graph mode")
                         .short("g")
                         .long("graph")
                         )
                    .arg(Arg::with_name("verbose")
                         .help("verbose mode")
                         .short("v")
                         .long("verbose")
                         )
                    .arg(Arg::with_name("ref")
                         .help("improved git-reflog")
                         .short("r")
                         .long("ref")
                         )
                    .arg(Arg::with_name("all")
                         .help("show all")
                         .short("a")
                         .long("all")
                         )
                    )
        .subcommand(SubCommand::with_name("diff")
                    .about("improved git-diff")
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
                    .about("rusgit add and rusgit commit")
                    .arg(Arg::with_name("files")
                         .help("victim files")
                         .required(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("message")
                         .help("commit message")
                         .short("m")
                         .long("message")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("improve")
                         .value_name("message")
                         .help("improve-prefix message")
                         .short("i")
                         .long("improve")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("implement")
                         .value_name("message")
                         .help("implement-prefix message")
                         .short("I")
                         .long("implement")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("remove")
                         .value_name("message")
                         .help("remove-prefix message")
                         .short("r")
                         .long("remove")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("refactor")
                         .value_name("message")
                         .help("refactor-prefix message")
                         .short("R")
                         .long("refactor")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("use")
                         .value_name("message")
                         .help("use-prefix message")
                         .short("u")
                         .long("use")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("update")
                         .value_name("message")
                         .help("update-prefix message")
                         .short("U")
                         .long("update")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("add")
                         .value_name("message")
                         .help("add-prefix message")
                         .short("a")
                         .long("add")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("change")
                         .value_name("message")
                         .help("change-prefix message")
                         .short("c")
                         .long("change")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("fix")
                         .value_name("message")
                         .help("fix-prefix message")
                         .short("f")
                         .long("fix")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("support")
                         .value_name("message")
                         .help("support-prefix message")
                         .short("s")
                         .long("support")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("allow")
                         .value_name("message")
                         .help("allow-prefix message")
                         .short("l")
                         .long("allow")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("avoid")
                         .value_name("message")
                         .help("avoid-prefix message")
                         .short("v")
                         .long("avoid")
                         .takes_value(true)
                         .multiple(true)
                         )
                    )
        .subcommand(SubCommand::with_name("push")
                    .about("improved git-push")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         )
                    )
        .subcommand(SubCommand::with_name("branch")
                    .about("improved git-branch")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         .index(1)
                         )
                    .arg(Arg::with_name("remote-branch")
                         .help("remote branch name")
                         .index(2)
                         )
                    .arg(Arg::with_name("delete")
                         .help("delete branch")
                         .short("d")
                         .long("delete")
                         .takes_value(true)
                         .value_name("branch")
                         )
                    .arg(Arg::with_name("DELETE")
                         .help("force delete branch")
                         .short("D")
                         .long("DELETE")
                         .takes_value(true)
                         .value_name("branch")
                         )
                    .arg(Arg::with_name("remote-delete")
                         .help("delete remote branch")
                         .long("remote-delete")
                         .takes_value(true)
                         .value_name("branch")
                         )
                    )
        .subcommand(SubCommand::with_name("pull")
                    .about("improved git-pull")
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
                    .about("improved git-merge")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         .required(true)
                         )
                    )
        .subcommand(SubCommand::with_name("rebase")
                    .about("improved git-rebase")
                    .arg(Arg::with_name("branch")
                         .help("branch name")
                         .required_unless("i")
                         )
                    .arg(Arg::with_name("i")
                         .help("rebase -i")
                         .short("i")
                         .takes_value(true)
                         .value_name("commit-id")
                         )
                    )
        .subcommand(SubCommand::with_name("clone")
                    .about("improved git-clone")
                    .arg(Arg::with_name("repo")
                         .help("user-name/repository-name")
                         .required(true)
                         .index(1)
                         )
                    .arg(Arg::with_name("directory")
                         .help("dest directory name")
                         .index(2)
                         )
                    )
        .subcommand(SubCommand::with_name("undo")
                    .about("undo actions")
                    .subcommand(SubCommand::with_name("commit")
                                .about("undo commit(keep changes)")
                                .arg(Arg::with_name("hard")
                                     .help("undo commit(not keep changes)")
                                     .long("hard")
                                     )
                                .arg(Arg::with_name("commit id")
                                     .help("undo change until <commit id>")
                                     .long("id")
                                     .takes_value(true)
                                     )
                                )
                    .subcommand(SubCommand::with_name("add")
                                .about("undo adding stage")
                                .arg(Arg::with_name("file")
                                     .help("file name")
                                     .takes_value(true)
                                     )
                                )
                    .subcommand(SubCommand::with_name("head")
                                .about("undo changes")
                                .arg(Arg::with_name("file")
                                     .help("file name")
                                     .takes_value(true)
                                     .required(true)
                                     )
                                )
                    .subcommand(SubCommand::with_name("orig")
                                .about("go to ORIG_HEAD")
                                .arg(Arg::with_name("hard")
                                     .help("not keep changes")
                                     .long("hard")
                                     )
                                )
                    )
        .subcommand(SubCommand::with_name("tag")
                    .about("improved git-tag")
                    .arg(Arg::with_name("message")
                         .help("messaging tag")
                         .short("m")
                         .long("message")
                         .takes_value(true)
                         .multiple(true)
                         )
                    .arg(Arg::with_name("editor")
                         .help("open editor")
                         .short("e")
                         .long("editor")
                         )
                    .arg(Arg::with_name("delete")
                         .help("delete tag")
                         .short("d")
                         .long("delete")
                         )
                    .arg(Arg::with_name("show")
                         .help("git show")
                         .short("s")
                         .long("show")
                         .value_name("tag-name")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("tag-name")
                         .help("tag name")
                         .takes_value(true)
                         )
                    )
        .subcommand(SubCommand::with_name("complete")
                    .about("completiton subcommand")
                    .arg(Arg::with_name("ac")
                         .help("ac-subcommand alias")
                         .long("ac")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("add")
                         .help("add-subcommand alias")
                         .long("add")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("diff")
                         .help("diff-subcommand alias")
                         .long("diff")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("branch")
                         .help("branch-subcommand alias")
                         .long("branch")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("merge")
                         .help("merge-subcommand alias")
                         .long("merge")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("pull")
                         .help("pull-subcommand alias")
                         .long("pull")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("push")
                         .help("push-subcommand alias")
                         .long("push")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("rebase")
                         .help("rebase-subcommand alias")
                         .long("rebase")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("undo")
                         .help("undo-subcommand alias")
                         .long("undo")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("tag")
                         .help("tag-subcommand alias")
                         .long("tag")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("log")
                         .help("log-subcommand alias")
                         .long("log")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("commit")
                         .help("commit-subcommand alias")
                         .long("commit")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("clone")
                         .help("clone-subcommand alias")
                         .long("clone")
                         .takes_value(true)
                         )
                    .arg(Arg::with_name("status")
                         .help("status-subcommand alias")
                         .long("status")
                         .takes_value(true)
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
        "rebase" => rebase_trigger(&matches),
        "ac" => ac_trigger(&matches),
        "clone" => clone(&matches),
        "undo" => undo(&matches),
        "tag" => tag_trigger(&matches),
        "complete" => complete(&matches),
        _ => help()
    } 
}
