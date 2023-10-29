use tiktoken_rs::CoreBPE;
use crate::ctags::{Ctag, CtagsOutput};

pub trait TokensLen {
    fn token_len(&self, bpe: &CoreBPE) -> usize;
}

impl TokensLen for String {
    fn token_len(&self, bpe: &CoreBPE) -> usize {
        count_tokens(self, bpe)
    }
}

impl TokensLen for Ctag {
    fn token_len(&self, bpe: &CoreBPE) -> usize {
        count_tokens(&serde_json::to_string(self).unwrap(), bpe)
    }
}

impl TokensLen for CtagsOutput {
    fn token_len(&self, bpe: &CoreBPE) -> usize {
        self.0.iter().fold(0, |acc, t| acc + t.token_len(bpe))
    }
}

fn count_tokens(s: &str, bpe: &CoreBPE) -> usize {
    bpe.encode_with_special_tokens(s).len()
}
