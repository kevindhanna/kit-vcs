extern crate clap;
use std::env;
use std::io::Result;
use clap::{Arg, App, SubCommand};
mod commands;

pub fn main() -> Result<()> {
    let dir = env::current_dir()?;

    let matches = App::new("Kit - a Git clone")
        .version("1.0")
        .author("Kevin H. <kevindhanna@live.com>")
        .about("Clone of Git just because")
        .subcommand(SubCommand::with_name("add")
                    .about("Add changes to the staging area")
                    .arg(Arg::with_name("FILE")
                         .help("The file to add")
                         .required(true)))
        .subcommand(SubCommand::with_name("cat-file")
                    .arg(Arg::with_name("OBJECT")
                         .help("The controlled object")
                         .required(true)))
        .subcommand(SubCommand::with_name("checkout")
                    .arg(Arg::with_name("REF")
                         .help("Check out a branch/ref")
                         .required(true)))
        .subcommand(SubCommand::with_name("commit")
                    .arg(Arg::with_name("message")
                         .value_name("Message")
                         .short("m")
                         .help("Commit message")
                         .required(true)))
        .subcommand(SubCommand::with_name("hash-object")
                    .arg(Arg::with_name("FILE")
                         .help("File to hash?")
                         .required(true)))
        .subcommand(SubCommand::with_name("init")
                    .help("Initialize a new, empty repository.")
                    .arg(Arg::with_name("PATH")
                         .default_value(".")
                         .help("Where to create the repository.")))
        .subcommand(SubCommand::with_name("log")
             .help("Show commit history for the current ref"))
        .subcommand(SubCommand::with_name("ls-tree")
             .help("not sure"))
        .subcommand(SubCommand::with_name("merge")
                    .arg(Arg::with_name("REF")
                         .help("Merge another branch into the current branch")
                         .required(true)))
        .subcommand(SubCommand::with_name("rebase")
                    .arg(Arg::with_name("REF")
                         .help("Replay the changes from this branch onto another branch")
                         .required(true)))
        .arg(Arg::with_name("rev-parse")
             .value_name("REF")
             .takes_value(true)
             .help("Unsure"))
        .subcommand(SubCommand::with_name("rm")
                    .arg(Arg::with_name("REF")
                         .help("Remove a file from source tracking")
                         .required(true)))
        .subcommand(SubCommand::with_name("show-ref")
                    .arg(Arg::with_name("FILE")
                         .help("Show a ref?")
                         .required(true)))
        .subcommand(SubCommand::with_name("tag")
                    .arg(Arg::with_name("FILE")
                         .help("Donno")
                         .required(true)))
        .get_matches();

    let mut args: Vec<&str> = Vec::new();
    for arg in ["rm", "show-ref", "tag"].iter() {
        if matches.is_present(arg) {
            println!("Got {}!", arg);
            args.push(arg)
        }
    }

    if args.is_empty() {
        match matches.subcommand_name() {
            Some("add") => println!("Git add was used"),
            Some("cat-file") => {
                if let Some(args) = matches.subcommand_matches("cat-file") {
                    commands::cat_file(dir, args)?
                };
            }
            Some("checkout") => println!("Git checkout was used"),
            Some("commit") => println!("Git commit was used"),
            Some("hash-object") => println!("Git hash-object was used"),
            Some("init") => {
                if let Some(args) = matches.subcommand_matches("init") {
                    commands::init(dir, args)?
                };
            },
            Some("log") => println!("Git log was used"),
            Some("ls-tree") => println!("Git ls-tree was used"),
            Some("merge") => println!("Git merge was used"),
            Some("rebase") => println!("Git rebase was used"),
            Some("rm") => println!("Git rm was used"),
            Some("show-ref") => println!("Git show-ref was used"),
            Some("tag") => println!("Git tag was used"),
            None => println!("Fuck you"),
            _ => unreachable!()
        }
    };
    Ok(())
}
