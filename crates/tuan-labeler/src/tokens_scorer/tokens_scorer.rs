use super::corpus::Corpus;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct TokensScorer {
    tokenize: Arc<dyn Fn(&Path) -> Result<Vec<String>, std::io::Error> + Send + Sync>,
    tokens: HashMap<PathBuf, Vec<String>>,
}

impl TokensScorer {
    pub(crate) fn new<
        T: Fn(&Path) -> Result<Vec<String>, std::io::Error> + Send + Sync + 'static,
    >(
        tokenize: T,
    ) -> Self {
        let tokenize = Arc::new(tokenize);
        let tokens = HashMap::new();
        TokensScorer { tokenize, tokens }
    }

    pub(crate) fn ingest(&mut self, files: &Vec<PathBuf>) {
        measure_time::info_time!("Tokenizing");
        let tokens = files
            .par_iter()
            .map(|file_path| {
                let toks = (self.tokenize)(file_path);
                (file_path.clone(), toks.unwrap_or_default())
            })
            .collect::<HashMap<_, _>>();
        self.tokens = tokens;
    }

    pub fn run(&self, selected_files: &Vec<PathBuf>) -> Result<Vec<(String, f32)>, std::io::Error> {
        let selected_tokens = {
            measure_time::info_time!("Selecting tokens");
            selected_files
                .iter()
                .filter_map(|p| self.tokens.get(p).map(|toks| (p, toks)))
                .collect::<HashMap<_, _>>()
        };

        let corpus = {
            measure_time::info_time!("Building corpus");
            let mut corpus = Corpus::default();
            for (_, tokens) in &self.tokens {
                corpus.ingest_document(&tokens);
            }
            corpus
        };

        let scores = {
            measure_time::info_time!("Computing TF-IDF scores for selected file paths");
            let all_tokens: Vec<String> = selected_tokens
                .values()
                .flat_map(|toks| *toks)
                .cloned()
                .collect();
            corpus.tfidf(&all_tokens)
        };

        Ok(scores)
    }
}
