use crate::config::CONFIG;
use dotenv::dotenv;
use env_logger::Env;
use log::{debug, trace};
use openai_macros::{ai_agent, message};
use openai_utils::api_key;
use std::env;
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_json::from_str;
use tiktoken_rs::cl100k_base;
use walkdir::DirEntry;
use crate::ctags::CtagsOutput;

mod config;
mod ctags;
mod macros;
mod tiktoken;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    trace!("env_logger has been set up");
    dotenv()?;
    trace!("dotenv has been set up");
    api_key(env::var("OPENAI_API_KEY")?);
    trace!("openai_api_key has been set has been set up");

    let blacklist = blacklist().await?;

    let tags: CtagsOutput = CtagsOutput::get_tags(&as_paths(&blacklist)).tags();

    let bpe = cl100k_base()?;

    let readable = tags.max_slices(&bpe, 4096);

    // debug!("{readable:#?}");

    Ok(())
}

fn get_files(input: Vec<impl Display>) -> anyhow::Result<Vec<DirEntry>> {
    let project_dir = CONFIG.read().unwrap().clone().repo_location.unwrap();
    let walkdir = walkdir::WalkDir::new(project_dir);

    let filter = |e: &DirEntry| {
        for n in input.iter().map(|s| s.to_string()) {
            if e.file_name().to_str().unwrap() == n {
                return false;
            }
        }
        true
    };

    let res = walkdir
        .into_iter()
        .filter_entry(filter)
        .filter(|e| !e.as_ref().unwrap().file_type().is_dir())
        .collect::<Result<Vec<_>, walkdir::Error>>()
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;

    debug!("project files: {res:#?}");
    Ok(res)
}

fn get_root_entries() -> anyhow::Result<Vec<String>> {
    let project_dir = CONFIG.read().unwrap().clone().repo_location.unwrap();
    let read_dir = std::fs::read_dir(project_dir)?;

    let mut entries = Vec::new();
    for entry in read_dir {
        let entry = entry?;
        entries.push(String::from(entry.file_name().to_str().unwrap()));
    }

    trace!("root entries: {entries:#?}");

    Ok(entries)
}

async fn blacklist() -> anyhow::Result<Vec<PathBuf>> {
    let project_summary = CONFIG.read().unwrap().project_summary.clone();

    // Get root level entries
    let root_entries = get_root_entries()?;

    let agent = ai_agent! {
        model: "gpt-3.5-turbo",
        temperature: 0.0,
        system_message: "Your job is to filter paths that contain build files from the root directory. You have to respond in a JSON array format.",
        messages: [
            message!(system, user: "example_input", content: r#"[
              "app.js",
              "dist",
              "build",
              "package.json",
              "README.md",
              "public",
              "views",
              "routes",
              "models",
              "controllers",
              "config",
              "tests",
              "node_modules"
            ]"#),
            message!(system, user: "example_response", content: r#"["node_modules", "dist", "build"]"#),
            message!(user, content: format!("{}", serde_json::to_string_pretty(&root_entries).unwrap())),
        ],
    };

    let chat = agent.create().await?;
    let res = chat.choices[0].message.content.clone().unwrap();
    let res = serde_json::from_str(&res).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    debug!("blacklist: {res:#?}");
    Ok(res)

}


fn as_paths(v: &Vec<PathBuf>) -> Vec<&Path> {
    v.iter().map(PathBuf::as_path).collect::<Vec<_>>()
}