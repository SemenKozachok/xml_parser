use std::fmt;
use xml_parser::{XmlNode, ParseError};

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(CliError::MissingArgs("no command provided"));
    }

    match args[1].as_str() {
        "parse" => handle_parse(&args)?,
        "help" | "-help" => print_help(),
        "credits" => print_credits(),
        cmd => return Err(CliError::UnknownCommand(cmd.to_string())),
    }

    Ok(())
}


#[derive(Debug)]
enum CliError {
    MissingArgs(&'static str),
    UnknownCommand(String),
    Parse(ParseError),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::MissingArgs(message) => {
                write!(f, "Missing argument: {}.\nType -help for more information.", message)
            }
            CliError::UnknownCommand(command) => {
                write!(f, "Unknown command: '{}'.\nType -help for more information.", command)
            }
            CliError::Parse(e) => write!(f, "{}.\nType -help for more information.", e),
        }
    }
}

impl From<ParseError> for CliError {
    fn from(err: ParseError) -> Self {
        CliError::Parse(err)
    }
}

fn handle_parse(args: &[String]) -> Result<(), CliError> {
    if args.len() < 3 {
        return Err(CliError::MissingArgs("path to XML file"));
    }

    let path = &args[2];
    let tree = XmlNode::from_path(path)?;

    if args.len() == 3 {
        println!("{}", tree);
        return Ok(());
    }

    match args[3].as_str() {
        "-get" => {
            let tag = args.get(4).ok_or(CliError::MissingArgs("tag name for -get"))?;
            match tree.get_contents_of(tag) {
                Some(content) => println!("Found <{}> is : {}", tag, content),
                None => println!("No <{}> node found.", tag),
            }
        }
        "-get_all" => {
            let tag = args.get(4).ok_or(CliError::MissingArgs("tag name for -get_all"))?;
            let results = tree.get_nodes(tag)
                            .iter()
                            .map(|node| node.content.clone()).collect::<Vec<String>>();

            println!("Found {} <{}> tag(s):", results.len(), tag);
            for (i, item) in results.iter().enumerate() {
                match item {
                    content if !content.is_empty() => println!("{}. {}", i + 1, content),
                    _ => println!("{}. None", i + 1),
                }
            }
        }
        cmd => return Err(CliError::UnknownCommand(cmd.to_string())),
    }
    Ok(())
}



fn print_help() {
    println!(
        r#"
XML Parser CLI

Usage:
  parse <path/to/file>                Parse XML file and print its tree.
  parse <path/to/file> -get [tag]     Find and print contents of first node with given tag.
  parse <path/to/file> -get_all [tag] Find and list contents of all nodes with given tag.

Other commands:
  help, -help        Show this help message.
  credits            Show authorship information.

To Run tests:
  cargo test
"#
    );
}


fn print_credits() {
    println!(
        r#"
XML Parser with pest
Author: Semen Kozachok
Year: 2025
Developed as a simple educational XML parser in Rust.
"#
    );
}
