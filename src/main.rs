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

fn status_with_ls(mode: &str, ls: &str) {
    process(&ls);
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
    let files: Vec<&str> = matches.subcommand_matches("add").unwrap().values_of("files").unwrap().collect();
    add(files);
    status("");
}

fn commit(message: &str) {
    if message == "" {
        process("git commit 1> /dev/null");
    } else {
        process(["git commit -m \"", message, "\" 1> /dev/null"].join(" ").as_str());
    }
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

fn log(num: i32, verbose: &str) {
    if verbose == "verbose" {
        process(["git log --decorate=short -p -", num.to_string().as_str()].join("").as_str());
    } else {
        process(["git log --decorate=short --oneline -", num.to_string().as_str()].join("").as_str());
    }
}
fn log_graph(num: i32, verbose: &str) {
    if verbose == "verbose" {
        process(["git log --decorate=short --graph -p -", num.to_string().as_str()].join("").as_str());
    } else {
        process(["git log --decorate=short --graph --oneline -", num.to_string().as_str()].join("").as_str());
    }
}

fn log_trigger(matches: &clap::ArgMatches) {
    let verbose = if matches.subcommand_matches("log").unwrap().is_present("verbose") {
        "verbose"
    } else {
        ""
    };
    if matches.subcommand_matches("log").unwrap().is_present("num") {
        if matches.subcommand_matches("log").unwrap().is_present("graph") {
            log_graph(matches.subcommand_matches("log").unwrap().value_of("num").unwrap().parse().unwrap(), verbose);
        } else {
            log(matches.subcommand_matches("log").unwrap().value_of("num").unwrap().parse().unwrap(), verbose);
        }
    } else {
        if matches.subcommand_matches("log").unwrap().is_present("graph") {
            log_graph(3, verbose);
        } else {
            log(3, verbose);
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

fn ac(files: Vec<&str>, message: &str) {
    add(files);
    commit(message);
}

fn ac_trigger(matches: &clap::ArgMatches) {
    let files: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("files").unwrap().collect();
    let message: String = if matches.subcommand_matches("ac").unwrap().is_present("message") {
        let message: Vec<&str> = matches.subcommand_matches("ac").unwrap().values_of("message").unwrap().collect();
        let message: String = message.join(" ");
        message
    } else {
        "".to_string()
    };
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
    let branch_name = matches.subcommand_matches("rebase").unwrap().value_of("branch").unwrap();
    rebase(branch_name);
}

fn rebase(branch_name: &str) {
    process([
            "git rebase",
            branch_name
    ].join(" ").as_str());
}

fn alias() {
    println!("\
alias rs=\"rusgit status --ls 'ls --color=always'\"
alias ra=\"rusgit add\"
alias rc=\"rusgit commit\"
alias rac=\"rusgit ac\"
alias rl=\"rusgit log\"
alias rd=\"rusgit diff\"
alias rb=\"rusgit branch\"
alias ru=\"rusgit undo\"
alias rt=\"rusgit tag\"
alias rpush=\"rusgit push\"
alias rpull=\"rusgit pull\"
alias rmerge=\"rusgit merge\"
alias rrebase=\"rusgit rebase\"
alias rclone=\"rusgit clone\"\
");
}

fn clone(matches: &clap::ArgMatches) {
    process([
            "git clone ",
            "https://github.com/",
            matches.subcommand_matches("clone").unwrap().value_of("repo").unwrap()
    ].join("").as_str());
}

fn undo(matches: &clap::ArgMatches) {
    if matches.subcommand_matches("undo").unwrap().is_present("commit") {
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
            let result = system_allow_stderr(["git checkout ",
                    commit_id
            ].join("").as_str());
            for (i, x) in result.stderr.split("\n").enumerate() {
                if i == 0 && x.contains("Note: checking out") {
                    continue;
                } else if i == 1 && x == "" { 
                    continue;
                } else if i == 2 && x.contains("state. You can look around, make experimental") {
                    continue;
                } else if i == 3 && x.contains("changes and commit them, and you can discard any commits you make in this") {
                    continue;
                } else if i == 4 && x.contains("state without impacting any branches by performing another checkout.") {
                    continue;
                } else if i == 5 && x.contains("") {
                    continue;
                } else if i == 6 && x.contains("If you want to create a new branch to retain commits you create, you may") {
                    continue;
                } else if i == 7 && x.contains("by using -b with the checkout command again. Example:") {
                    continue;
                } else if i == 8 && x.contains("") {
                    continue;
                } else if i == 9 && x.contains("git checkout -b") {
                    continue;
                } else if i == 10 && x.contains("") {
                    continue;
                } else if i == 11 && x.contains("HEAD is now at") {
                    continue;
                } else if i < 12 {
                    println!("{}", result.stderr);
                    std::process::exit(0);
                }
            }
            println!("Success.");
            println!("Next.. Please create new branch.");
            println!("\ntry:");
            println!("  rusgit branch <new-branch>");
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
        std::process::exit(0);
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

fn main() {
    let matches = App::new("rusgit")
        .version("0.1.0")
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
                         .required(true)
                         .multiple(true)
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
                    .arg(Arg::with_name("num")
                         .help("num of logs")
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
                         .required(true)
                         )
                    )
        .subcommand(SubCommand::with_name("alias")
                    .about("print aliases")
                    )
        .subcommand(SubCommand::with_name("clone")
                    .about("improved git-clone")
                    .arg(Arg::with_name("repo")
                         .help("repository name")
                         .required(true)
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
        "alias" => alias(),
        "clone" => clone(&matches),
        "undo" => undo(&matches),
        "tag" => tag_trigger(&matches),
        _ => help()
    } 
}
