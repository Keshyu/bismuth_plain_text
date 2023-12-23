mod parser;

use anyhow::{Context, Result};
use dbg_pls::pretty as fmt_pretty;
use std::{env, fs};

fn main() -> Result<()> {
    let file_name = env::args().nth(1).context("file name not provided")?;
    let source = fs::read_to_string(&file_name)?;
    let tokens = parser::lex(&source.chars().collect::<Box<_>>())?;
    let expression = parser::parse(&tokens)?;

    println!("{}", fmt_pretty(&expression));

    Ok(())
}
