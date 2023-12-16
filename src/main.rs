use std::fs::File;
use std::io::{prelude::*, self};
use anyhow::{Context, Result};
use eyre;

fn header() -> Result<()>{
    let mut file = File::open("hello.txt.lz4")?;
    Ok(())
}

fn main() {
    header().unwrap();
    println!("Hello, world!");
}
