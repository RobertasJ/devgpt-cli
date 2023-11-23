use std::path::PathBuf;
use log::{debug, trace};
use openai_macros::{ai_agent, message};
use serde_json::from_str;
use crate::config::CONFIG;

pub mod search;

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

pub async fn blacklist() -> anyhow::Result<Vec<PathBuf>> {
    let project_summary = CONFIG.read().unwrap().project_summary.clone();
    
    // Get root level entries
    let root_entries = get_root_entries()?;
    
    let agent = ai_agent! {
        model: "gpt-4-1106-preview",
        temperature: 0.0,
        system_message: "Your job is to filter paths that contain build files from the root directory. You have to respond in a JSON array format. DO NOT FILTER OUT CONFIG OR SOURCE FILES. remember to not include anything before or after the array, your answer will have to be parsed by a computer.",
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
    let res = from_str(&res).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    debug!("blacklist: {res:#?}");
    Ok(res)
    
}
