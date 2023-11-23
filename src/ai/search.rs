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
    
    let mut result = CtagsOutput::default();

    finder.push_function(|args: FindNameArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
        result.0.extend(tags.0.iter().filter(|t| t.name_contains(&args.name)).cloned());
    });
    finder.push_function(|args: FindPathArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
        result.0.extend(tags.0.iter().filter(|t| t.path_is(&args.path)).cloned());
    });
    finder.push_function(|args: FindKindArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
        result.0.extend(tags.0.iter().filter(|t| t.kind_contains(&args.kind)).cloned());
    });
    finder.push_function(|args: FindLineRangeArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&BLACKLIST));
        result.0.extend(tags.0.iter().filter(|t| {
            if let Some(line) = t.line {
                line >= args.from && line <= args.to
            } else {
                false
            }
        }).cloned());
    });
    
    todo!()
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag name contains the given name as argument")]
struct FindNameArgs {
    #[schemars(description = "The name to check if contained")]
    name: String,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag path is the path specified")]
struct FindPathArgs {
    #[schemars(description = "The path specified")]
    path: PathBuf,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if the tag kind contains the given kind as argument")]
struct FindKindArgs {
    #[schemars(description = "The kind to check if contained")]
    kind: String,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Checks if a tag is within an inclusive range")]
struct FindLineRangeArgs {
    #[schemars(description = "the start of the range")]
    from: u32,
    #[schemars(description = "the end of the range")]
    to: u32,
}

#[cfg(test)]
mod tests {

}
