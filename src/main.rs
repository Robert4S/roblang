#![allow(warnings)]

use clap::Parser;

use crate::lexing::*;
use crate::parsing::*;

mod parsing;
mod lexing;
mod generation;

#[derive(Parser, Debug)]
/// Compiler for the roblang language. Everything is broken, nothing works, and all you get are some unlinked, possibly incorrect parse tree nodes.
#[command(version, about, long_about = None)]
struct Args {
    /// Build mode. Options are: "build", "run"
    #[arg(short, long)]
    mode: String,

    /// Roblang source file to compile
    #[arg(short, long)]
    file: String,
}

fn main() {
    let args = Args::parse();
    let run = (args.mode == "run");
    let buildfile = args.file;
    build(&buildfile);
    if run {
        let mut namechars = buildfile.chars().peekable();
        let mut newname = String::new();
        'whilelet: while let Some(current) = namechars.next() {
            if current == '.' && namechars.next().is_some() && (namechars.next().unwrap() != '/') {
                break 'whilelet;
            }
            newname.push(current);
        }
        let robjstring = ".robj".to_string();
        newname = newname + &robjstring;

        let process = std::process::Command::new("python3")
            .arg(newname.clone())
            .spawn()
            .expect("could not run");
        return;
    }
}

fn build(file: &String) {
    let tokens = tokenize::parse_start(&file).expect("a problem magically appeared");
    for i in tokens.clone() {
        // println!("{:?}", i);
    }
    let mut parser = parsetree::ParseTree::new(&tokens);
    let mut root = parser.parse();
    nodes::print_program(&root.children, 0)
}


