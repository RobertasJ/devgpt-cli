use dotenv::dotenv;
use env_logger::Env;
use log::{debug, trace};
use openai_utils::api_key;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{stdin, Write};
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
use crate::ai::blacklist;
use crate::ai::search::find_file;
use crate::ctags::{Ctag, CtagsOutput};

mod config;
mod ctags;
mod macros;
mod tiktoken;
mod ai;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    trace!("env_logger has been set up");
    dotenv()?;
    trace!("dotenv has been set up");
    api_key(env::var("OPENAI_API_KEY")?);
    trace!("openai_api_key has been set has been set up");
    
    let blacklist = blacklist().await.unwrap();
    
    let mut buf = Default::default();
    
    println!("enter a search:");
    stdin().read_line(&mut buf)?;

    let file = find_file(&buf, blacklist).await;
    
    println!("{file:#?}");

    Ok(())
}

fn as_paths(v: &Vec<PathBuf>) -> Vec<&Path> {
    v.iter().map(PathBuf::as_path).collect::<Vec<_>>()
}


