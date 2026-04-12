//! 中文分词 Tokenizer — 基于 jieba-rs

use std::sync::LazyLock;
use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

use jieba_rs::Jieba;

/// 全局 Jieba 实例（加载字典较慢，只初始化一次）
static JIEBA: LazyLock<Jieba> = LazyLock::new(Jieba::new);

/// Jieba 中文分词器
#[derive(Clone, Default)]
pub struct JiebaTokenizer;

impl Tokenizer for JiebaTokenizer {
    type TokenStream<'a> = JiebaTokenStream;

    fn token_stream<'a>(&mut self, text: &'a str) -> Self::TokenStream<'a> {
        let tokens = JIEBA.tokenize(text, jieba_rs::TokenizeMode::Search, false);
        let tokens: Vec<Token> = tokens
            .iter()
            .map(|t| Token {
                offset_from: t.start,
                offset_to: t.end,
                text: t.word.to_string(),
                position: t.start,
                position_length: t.end - t.start,
            })
            .collect();
        JiebaTokenStream {
            tokens,
            index: 0,
        }
    }
}

pub struct JiebaTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for JiebaTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::tokenizer::Tokenizer;

    #[test]
    fn test_jieba_chinese() {
        let mut tokenizer = JiebaTokenizer;
        let mut stream = tokenizer.token_stream("我是一个程序员");
        let mut tokens = Vec::new();
        while stream.advance() {
            tokens.push(stream.token().text.clone());
        }
        assert!(!tokens.is_empty());
        assert!(tokens.contains(&"程序员".to_string()));
    }

    #[test]
    fn test_jieba_mixed() {
        let mut tokenizer = JiebaTokenizer;
        let mut stream = tokenizer.token_stream("Rust编程语言");
        let mut tokens = Vec::new();
        while stream.advance() {
            tokens.push(stream.token().text.clone());
        }
        assert!(!tokens.is_empty());
    }
}
