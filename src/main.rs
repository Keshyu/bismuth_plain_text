use anyhow::{Context, Result};
use dbg_pls::DebugPls;
use std::{env, fs};
use tree_sitter::{Parser, Tree};

fn main() -> Result<()> {
    let file_name = env::args().nth(1).context("file name not provided")?;
    let source = fs::read_to_string(&file_name)?;
    let expression = parse(&source)?;

    println!("{:?}", &expression);

    Ok(())
}

fn parse(source: impl AsRef<[u8]>) -> Result<Tree> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_bismuth::language())
        .context("setting Bismuth as the parsing language")?;
    Ok(parser.parse(source, None).unwrap())
}

#[derive(DebugPls)]
pub enum Expression {
    Group(Box<[Expression]>),
    Name(Box<str>),
}
