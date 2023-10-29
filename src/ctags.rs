use std::fmt::format;
use std::path::{Path, PathBuf};
use crate::config::CONFIG;
use crate::tiktoken::TokensLen;
use log::{debug, trace};
use serde::de::Error;
use serde::de::Unexpected::Str;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::process::Command;
use indicatif::{ProgressBar, ProgressStyle};
use tiktoken_rs::CoreBPE;

#[derive(Serialize, Debug, Clone)]
pub struct CtagsOutput(pub Vec<Ctag>);
#[derive(Serialize, Debug, Clone)]
pub enum Ctag {
    /// PseudoTag
    Ptag {
        name: String,
        path: String,
        pattern: Option<String>,
        /// is the language parsed
        parser_name: Option<String>,
    },
    Tag {
        name: String,
        path: String,
        pattern: Option<String>,
        kind: String,
        scope: Option<String>,
        scope_kind: Option<String>,
    },
}

#[derive(Deserialize)]
struct TempCtag {
    _type: String,
    name: String,
    path: String,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    parser_name: Option<String>,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    scope_kind: Option<String>,
}

impl<'de> Deserialize<'de> for Ctag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let temp = TempCtag::deserialize(deserializer)?;

        match temp._type.as_str() {
            "ptag" => Ok(Ctag::Ptag {
                name: temp.name,
                path: temp.path,
                pattern: temp.pattern,
                parser_name: temp.parser_name,
            }),
            "tag" => Ok(Ctag::Tag {
                name: temp.name,
                path: temp.path,
                pattern: temp.pattern,
                kind: temp.kind.ok_or(D::Error::custom("missing kind for tag"))?,
                scope: temp.scope,
                scope_kind: temp.scope_kind,
            }),
            _ => Err(D::Error::custom("invalid _type")),
        }
    }
}

impl Ctag {
    pub fn is_ptag(&self) -> bool {
        match self {
            Ctag::Ptag { .. } => true,
            Ctag::Tag { .. } => false,
        }
    }

    pub fn is_tag(&self) -> bool {
        match self {
            Ctag::Ptag { .. } => false,
            Ctag::Tag { .. } => true,
        }
    }
}

impl CtagsOutput {
    pub fn get_tags(blacklist: &[&Path]) -> Self {
        let repo_location = CONFIG.read().unwrap().repo_location.clone().unwrap();

        let blacklist: Vec<String> = blacklist.iter().map(|p| format!("--exclude={}", p.display())).collect();

        #[cfg(target_family = "windows")]
        let mut proc = Command::new("ctags\\ctags.exe");
        #[cfg(target_family = "unix")]
        let mut proc = Command::new("ctags");

        let res = proc
            .args({
                let mut vec = vec![
                    "-R",
                    "--output-format=json",
                    "-f",
                    "-",

                ];

                vec.extend(blacklist.iter().map(String::as_str));
                vec.push(repo_location.to_str().unwrap());
                vec
            })
            .output()
            .unwrap();

        trace!("ctags done executing");

        let s = String::from_utf8(res.stdout).unwrap();

        let res = Self(
            s.lines()
                .map(|s| from_str(s.trim()).unwrap())
                .collect(),
        );

        debug!("generated {} tags", res.0.len());

        res
    }

    pub fn tags(self) -> Self {
        let input_len = self.0.len();

        let res = Self(self.0.into_iter().filter(Ctag::is_tag).collect());

        trace!("removed {} tags", input_len - res.0.len());

        res
    }

    pub fn ptags(self) -> Self {
        Self(self.0.into_iter().filter(Ctag::is_ptag).collect())
    }

    pub fn max_slice(mut self, bpe: &CoreBPE, max_tokens: usize) -> (Self, Self) {
        let mut total_len = 0;
        let mut taken = vec![];
        let mut rest = self.0.into_iter().peekable();

        while let Some(item) = rest.next_if(|item| total_len + item.token_len(bpe) <= max_tokens) {
            total_len += item.token_len(bpe);
            taken.push(item);
        }

        let max = CtagsOutput(taken);
        self.0 = rest.collect();

        (max, self)
    }

    pub fn max_slices(mut self, bpe: &CoreBPE, max_tokens: usize) -> Vec<Self> {
        let pb = ProgressBar::new(self.0.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})").unwrap()
            .progress_chars("#>-"));

        let mut slices = vec![];

        while !self.is_empty() {
            let (max, rest) = self.max_slice(bpe, max_tokens);
            pb.inc(max.0.len() as u64);
            slices.push(max);
            self = rest;
        }

        pb.finish_with_message("done");

        debug!("made {} slices", slices.len());

        slices
    }

    fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    fn is_not_empty(&self) -> bool {
        self.0.len() != 0
    }
}
