#![feature(async_fn_in_trait)]

use std::env;
use std::fmt::Display;
use std::io::Write;
use dotenv::dotenv;
use env_logger::Env;
use walkdir::DirEntry;
use crate::config::CONFIG;
use openai_utils::api_key;
use openai_macros::{ai_agent, message};
use log::{debug, trace};

mod config;
mod macros;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
    trace!("env_logger has been set up");
    dotenv()?;
    trace!("dotenv has been set up");
    api_key(env::var("OPENAI_API_KEY")?);
    trace!("openai_api_key has been set has been set up");
    let res = blacklist().await?;
    let _res = get_files(res)?;

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

async fn blacklist() -> anyhow::Result<Vec<String>> {
    let project_summary = CONFIG.read().unwrap().project_summary.clone();

    // Get root level entries
    let root_entries = get_root_entries()?;

    let agent = ai_agent! {
        model: "gpt-3.5-turbo",
        temperature: 0.0,
        system_message: "The user is going to give you the type of project and your job is to provide a list of files and folders that are not part of the code itself. You have to respond in a JSON array format.",
        messages: [
            message!(user, content: r#"a generic nodejs app: [
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
            message!(assistant, content: r#"["node_modules", "dist", "build"]"#),
            message!(user, content: format!("{}: {:?}", project_summary, root_entries)),
        ],
    };

    let chat = agent.create().await?;
    let res = chat.choices[0].message.content.clone().unwrap();
    let res = serde_json::from_str(&res).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    debug!("blacklist: {res:#?}");
    Ok(res)

}
