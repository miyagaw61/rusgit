extern crate clap;

use clap::{App, Arg, SubCommand};
use std::process::Command;

fn shell(command: &str) -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .expect("failed to execute process");
    return std::str::from_utf8(&output.stdout[..]).unwrap().to_string()
}

fn status() -> String {
    return shell("git status --short")
}

fn add(files: Vec<&str>) {
    shell(["git add", files.join(" ").as_str()].join(" ").as_str());
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

    if matches.is_present("status") {
        println!("{}", status());
    } else if let Some(ref matches) = matches.subcommand_matches("add") {
        let files: Vec<&str> = matches.values_of("files").unwrap().collect();
        add(files);
        println!("{}", status());
    }
}
