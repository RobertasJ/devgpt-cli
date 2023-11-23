use std::ops::Range;
use std::path::{Path, PathBuf};
use openai_macros::{ai_agent, message};
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use crate::ai::blacklist;
use crate::{as_paths, BLACKLIST};
use crate::config::CONFIG;
use crate::ctags::{Ctag, CtagsOutput};

struct Context {
    file: PathBuf,
    lines: Range<u64>,
    code: String,
}

pub async fn find_context(search: &str) -> Vec<Context> {
    // find file
    
    // truncate
    
    todo!()
}

async fn find_file(predicate: &str) -> PathBuf {
    // create ai agent with system and add functions for searching.
    let mut finder = ai_agent! {
        model: "gpt-4-1106-preview",
        system_message: include_str!("finder.md"),
        messages: message!(user, content: format!("predicate: {predicate}"))
    };
    
    finder.push_function(find_name);
    finder.push_function(find_kind);
    finder.push_function(find_path);
    finder.push_function(find_line_range);
    
    
    
    todo!()
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag name contains the given name as argument")]
struct FindNameArgs {
    #[schemars(description = "The name to check if contained")]
    name: String,
}

fn find_name(args: FindNameArgs) -> CtagsOutput {
    let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
    CtagsOutput(tags.0.iter().filter(|t| t.name_contains(&args.name)).collect())
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag path is the path specified")]
struct FindPathArgs {
    #[schemars(description = "The path specified")]
    path: PathBuf,
}

fn find_path(args: FindPathArgs) -> CtagsOutput {
    let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
    CtagsOutput(tags.0.iter().filter(|t| t.path_is(&args.path)).collect())
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag kind contains the given kind as argument")]
struct FindKindArgs {
    #[schemars(description = "The kind to check if contained")]
    kind: String,
}

fn find_kind(args: FindKindArgs) -> CtagsOutput {
    let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
    CtagsOutput(tags.0.iter().filter(|t| t.kind_contains(&args.kind)).collect())
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if a tag is within an inclusive range")]
struct FindLineRangeArgs {
    #[schemars(description = "the start of the range")]
    from: u32,
    #[schemars(description = "the end of the range")]
    to: u32,
}

fn find_line_range(args: FindLineRangeArgs) -> CtagsOutput {
    let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
    CtagsOutput(tags.0.iter().filter(|t| {
        if let Some(line) = t.line {
            line >= args.from && line <= args.to
        } else {
            false
        }
    }).collect())
}

#[cfg(test)]
mod tests {

}