use crate::config::CONFIG;
use crate::tiktoken::TokensLen;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, trace};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::path::{Path, PathBuf};
use std::process::Command;
use tiktoken_rs::CoreBPE;

#[derive(Serialize, Debug, Clone)]
pub struct CtagsOutput(pub Vec<Ctag>);

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Ctag {
    pub _type: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub parser_name: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub scope_kind: Option<String>,
    #[serde(default)]
    pub line: Option<u32>,
}

impl Ctag {
    pub fn is_ptag(&self) -> bool {
        match self._type.as_str() {
            "tag" => false,
            "ptag" => true,
            _ => unimplemented!()
        }
    }

    pub fn is_tag(&self) -> bool {
        match self._type.as_str() {
            "tag" => true,
            "ptag" => false,
            _ => unimplemented!()
        }
    }

    pub fn kind_contains(&self, kind: &str) -> bool {
        if let Some(f) = &self.kind {
            f.to_lowercase().contains(&kind.to_lowercase())
        } else {
            false
        }
    }

    pub fn path_is(&self, path: &Path) -> bool {
        if let Some(f) = &self.path {
            f == path
        } else {
            false
        }
    }

    pub fn name_contains(&self, name: &str) -> bool {
        if let Some(f) = &self.name {
            f.to_lowercase().contains(&name.to_lowercase())
        } else {
            false
        }
    }
}

impl CtagsOutput {
    pub fn get_tags(blacklist: &[&Path]) -> Self {
        let repo_location = CONFIG.read().unwrap().project_dir.clone().unwrap();

        let blacklist: Vec<String> = blacklist
            .iter()
            .map(|p| format!("--exclude={}", p.display()))
            .collect();

        #[cfg(target_family = "windows")]
        let mut proc = Command::new("ctags\\ctags.exe");
        #[cfg(target_family = "unix")]
        let mut proc = Command::new("ctags");

        let res = proc
            .args({
                let mut vec = vec![
                    "--languages=Rust,C,C++,C#,Java,JavaScript,Python,Ruby,Go,Kotlin,TypeScript,Elixir,Erlang,Haskell,Lua,Perl,PHP,PowerShell,SQL,Sh,Tcl,Asm,D,Fortran,Cobol,HTML,CSS,JavaProperties",
                    "--kinddef-C=d,devgpt,devgpt-comments",
                    "--regex-C=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-C++=d,devgpt,devgpt-comments",
                    "--regex-C++=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-C#=d,devgpt,devgpt-comments",
                    "--regex-C#=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Java=d,devgpt,devgpt-comments",
                    "--regex-Java=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-JavaScript=d,devgpt,devgpt-comments",
                    "--regex-JavaScript=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Python=d,devgpt,devgpt-comments",
                    "--regex-Python=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Ruby=d,devgpt,devgpt-comments",
                    "--regex-Ruby=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Go=d,devgpt,devgpt-comments",
                    "--regex-Go=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Rust=d,devgpt,devgpt-comments",
                    "--regex-Rust=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Kotlin=d,devgpt,devgpt-comments",
                    "--regex-Kotlin=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-TypeScript=d,devgpt,devgpt-comments",
                    "--regex-TypeScript=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Elixir=d,devgpt,devgpt-comments",
                    "--regex-Elixir=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Erlang=d,devgpt,devgpt-comments",
                    "--regex-Erlang=/%\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Haskell=d,devgpt,devgpt-comments",
                    "--regex-Haskell=/--\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Lua=d,devgpt,devgpt-comments",
                    "--regex-Lua=/--\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Perl=d,devgpt,devgpt-comments",
                    "--regex-Perl=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-PHP=d,devgpt,devgpt-comments",
                    "--regex-PHP=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-PowerShell=d,devgpt,devgpt-comments",
                    "--regex-PowerShell=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-SQL=d,devgpt,devgpt-comments",
                    "--regex-SQL=/--\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Sh=d,devgpt,devgpt-comments",
                    "--regex-Sh=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Tcl=d,devgpt,devgpt-comments",
                    "--regex-Tcl=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Asm=d,devgpt,devgpt-comments",
                    "--regex-Asm=/;\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-D=d,devgpt,devgpt-comments",
                    "--regex-D=/\\/\\/\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Fortran=d,devgpt,devgpt-comments",
                    "--regex-Fortran=/!\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-Cobol=d,devgpt,devgpt-comments",
                    "--regex-Cobol=/\\*\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--kinddef-HTML=d,devgpt,devgpt-comments",
                    "--regex-HTML=/<!--\\s*DEV:\\s*(.*?)\\s*-->/\\1/d/",
                    "--kinddef-CSS=d,devgpt,devgpt-comments",
                    "--regex-CSS=/\\*\\s*DEV:\\s*(.*?)\\s*\\*\\//\\1/d/",
                    "--kinddef-JavaProperties=d,devgpt,devgpt-comments",
                    "--regex-JavaProperties=/#\\s*DEV:\\s*([^\\n]*)/\\1/d/",
                    "--fields=+n",
                    "-R", "--output-format=json", "-f", "-"
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
                .map(|s| from_str::<Ctag>(s.trim()).unwrap())
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
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

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
