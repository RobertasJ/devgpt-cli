use dotenv::dotenv;
use env_logger::Env;
use log::{debug, trace};
use openai_utils::api_key;
use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
use crate::ai::blacklist;
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

    let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST)).tags();
    
    let file = File::create("tags.jsonl");
    println!("{tags:#?}");

    let s = "Hello there!";
    
    let chars = s.chars().collect::<Vec<_>>();
    
    Ok(())
}

fn as_paths(v: &Vec<PathBuf>) -> Vec<&Path> {
    v.iter().map(PathBuf::as_path).collect::<Vec<_>>()
}

lazy_static! {
    static ref BLACKLIST: Vec<PathBuf> = {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            blacklist().await.unwrap()
        })
    };
}

