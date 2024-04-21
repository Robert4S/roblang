#![allow(warnings)]

use clap::Parser;

use crate::generation::generator;
use crate::lexing::*;
use crate::parsing::*;

mod generation;
mod lexing;
mod parsing;

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
    let mut tokens = (args.mode == "tokens");
    let mut nodes = args.mode == "nodes";
    if args.mode == "fulldbg" {
        nodes = true;
        tokens = true;
    }
    let buildfile = args.file;
    let _ = build(&buildfile, tokens, nodes);
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

        return;
    }
}

fn build(file: &String, tokensshow: bool, nodes: bool) -> Option<()> {
    let robstd = std::env::var("ROBSTD").unwrap_or_default();
    let cvec = std::env::var("CVEC").unwrap_or_default();
    let tokens = tokenize::parse_start(&file).expect("weird")?;
    if tokensshow {
        for i in tokens.clone() {
            println!("{:?}", i);
        }
    }
    let mut parser = parsetree::ParseTree::new(&tokens);
    let mut root = parser.parse();
    if root.check_top().is_none() {
        return None;
    }
    let gen = generator::Generator::new(root.clone());
    let writeres = gen.write();
    match writeres {
        Err(_) => {
            eprintln!("generation failed");
        }
        Ok(_) => {
            eprintln!("Build success!");
        }
    }
    let dotpos = {
        let mut ind = 0;
        for (index, item) in file.chars().enumerate() {
            ind = index;
            if item == '.' {
                break;
            }
        }
        ind
    };
    let fstr = file.split_at(dotpos).0;
    let process = std::process::Command::new("gcc")
        .arg("out.c")
        .arg("-o")
        .arg(fstr)
        .arg(format!("-I{}", robstd))
        .arg(format!("-I{}", cvec))
        .arg(format!("-L{}", cvec))
        .arg("-lvec")
        .spawn()
        .expect("could not run");
    if nodes {
        nodes::print_program(&root.children, 0);
    }
    Some(())
}
