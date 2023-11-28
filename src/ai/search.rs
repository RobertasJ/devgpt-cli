use std::cell::RefCell;
use std::io::Write;
use std::io::stdout;
use std::ops::Range;
use std::path::{Path, PathBuf};
use log::{debug, trace};
use openai_macros::{ai_agent, message};
use openai_utils::{FunctionCall, NoArgs};
use schemars::JsonSchema;
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use crate::ai::blacklist;
use crate::{as_paths, print_chat};
use crate::config::CONFIG;
use crate::ctags::{Ctag, CtagsOutput};

struct Context {
    file: PathBuf,
    lines: Range<u64>,
    code: String,
}
pub async fn find_file(search: &str, blacklist: Vec<PathBuf>) -> anyhow::Result<Option<Vec<PathBuf>>> {
    // create ai agent with system and add functions for searching.
    let mut finder = ai_agent! {
        model: "gpt-4-1106-preview",
        system_message: include_str!("finder.md"),
        temperature: 0.0,
        messages: message!(user, content: format!("{search}"))
    };
    
    trace!("tags: {:#?}", CtagsOutput::get_tags(&as_paths(&blacklist)));

    let result = RefCell::new(CtagsOutput(vec![]));

    let find_name = |args: FindNameArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&blacklist));
        // Borrow `result` mutably and replace its contents
        *result.borrow_mut() = CtagsOutput(tags.0.iter().filter(|t| t.name_contains(&args.name)).cloned().collect());
    };
    
    finder.push_function(&find_name, "find_name");
    
    let find_path = |args: FindPathArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&blacklist));
        *result.borrow_mut() = CtagsOutput(tags.0.iter().filter(|t| t.path_is(&args.path)).cloned().collect());
    };
    
    finder.push_function(&find_path, "find_path");
    
    let find_kind = |args: FindKindArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&blacklist));
        result.borrow_mut().0.extend(tags.0.iter().filter(|t| t.kind_contains(&args.kind)).cloned());
    };
    
    finder.push_function(&find_kind, "find_kind");
    
    let find_line_range = |args: FindLineRangeArgs| {
        let tags = CtagsOutput::get_tags(&as_paths(&blacklist));
        *result.borrow_mut() = CtagsOutput(tags.0.iter().filter(|t| {
            if let Some(line) = t.line {
                line >= args.from && line <= args.to
            } else {
                false
            }
        }).cloned().collect());
    };
    
    finder.push_function(&find_line_range, "find_line_range");
    
    let mut stop = false;

    let mut response = None;


    while !stop {
        let mut stop_searching = |args: StopSearchingArgs| {
            response = args.predicate_path;
            stop = true;
        };

        finder.push_function(&stop_searching, "stop_searching");

        let mut receiver = finder.create_stream().await?;
        print_chat!(receiver);
        let res = receiver.construct_chat().await?;

        finder.push_message(res.choices[0].clone().message);

        if let Some(FunctionCall { name, arguments }) = res.choices[0].clone().message.function_call {
            debug!("Function call received: {} with arguments: {}", name, arguments);
            match name.as_str() {
                "find_name" => {
                    if let Ok(args) = from_str(&arguments) {
                        debug!("Executing find_name with args: {:?}", args);
                        find_name(args);
                        finder.push_message(message!(system, content: format!("search results: {:?}", result.borrow_mut().0)));
                    } else {
                        finder.push_message(message!(system, content: "could not parse find_name arguments"));
                    }
                },
                "find_path" => {
                    if let Ok(args) = from_str(&arguments) {
                        debug!("Executing find_path with args: {:?}", args);
                        find_path(args);
                        finder.push_message(message!(system, content: format!("search result: {}", to_string_pretty(&result.borrow().0)?)));
                    } else {
                        finder.push_message(message!(system, content: "could not parse find_path arguments"));
                    }
                },
                "find_kind" => {
                    if let Ok(args) = from_str(&arguments) {
                        debug!("Executing find_kind with args: {:?}", args);
                        find_kind(args);
                        finder.push_message(message!(system, content: format!("search result: {}", to_string_pretty(&result.borrow().0)?)));
                    } else {
                        finder.push_message(message!(system, content: "could not parse find_kind arguments"));
                    }
                },
                "find_line_range" => {
                    if let Ok(args) = from_str(&arguments) {
                        debug!("Executing find_line_range with args: {:?}", args);
                        find_line_range(args);
                        finder.push_message(message!(system, content: format!("search result: {}", to_string_pretty(&result.borrow().0)?)));
                    } else {
                        finder.push_message(message!(system, content: "could not parse find_line_range arguments"));
                    }
                },
                "stop_searching" => {
                    if let Ok(args) = from_str(&arguments) {
                        debug!("Executing stop_searching with args: {:?}", args);
                        stop_searching(args);
                        finder.push_message(message!(system, content: format!("search result: {}", to_string_pretty(&result.borrow().0)?)));
                    } else {
                        finder.push_message(message!(system, content: "could not parse stop_searching arguments"));
                    }
                },
                _ => {
                    debug!("Function not found: {}", name);
                    finder.push_message(message!(system, content: "function not found"));
                },
            }

            debug!("Result after function call: {:#?}", result.borrow().0);
        }
    }

    Ok(response)
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

#[derive(Default, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[schemars(description = "Run this function to stop the searching of the file where the predicate is found")]
struct StopSearchingArgs {
    #[schemars(description = "The file where the predicate is found, if not found set to none")]
    predicate_path: Option<Vec<PathBuf>>
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;
    use std::str::FromStr;
    use env_logger::Env;
    use openai_utils::api_key;
    use crate::ai::blacklist;
    use crate::ai::search::find_file;
    use crate::config::CONFIG;

    fn init() {
        env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();
        dotenv::dotenv().unwrap();
        api_key(env::var("OPENAI_API_KEY").unwrap());
        (*CONFIG.write().unwrap()).project_dir = Some(PathBuf::from_str("mock_project").unwrap());
    }

    #[tokio::test]
    async fn test_searching() {
        init();
        
        let blacklist = blacklist().await.unwrap();
        let res = find_file("Has a tag of kind devgpt", blacklist).await;

        dbg!(res);
    }
}
