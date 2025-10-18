use super::corpus::Corpus;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator as _};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub(crate) struct TokensScorer {
    tokenize: Arc<dyn Fn(&Path) -> Result<Vec<String>, std::io::Error> + Send + Sync>,
}

impl TokensScorer {
    pub fn new<T: Fn(&Path) -> Result<Vec<String>, std::io::Error> + Send + Sync + 'static>(
        tokenize: T,
    ) -> Self {
        let tokenize = Arc::new(tokenize);
        TokensScorer { tokenize }
    }

    pub fn run(
        &self,
        files: &Vec<&Path>,
        selected_files: &Vec<&Path>,
    ) -> Result<Vec<(String, f32)>, std::io::Error> {
        let tokens = {
            measure_time::info_time!("Tokenizing");
            files
                .par_iter()
                .map(|file_path| {
                    let path = file_path;
                    let toks = (self.tokenize)(path);
                    (file_path, toks.unwrap_or_default())
                })
                .collect::<HashMap<_, _>>()
        };

        let selected_tokens = {
            measure_time::info_time!("Selecting tokens");
            selected_files
                .iter()
                .filter_map(|p| tokens.get(p).map(|toks| (p, toks)))
                .collect::<HashMap<_, _>>()
        };

        let corpus = {
            measure_time::info_time!("Building corpus");
            let mut corpus = Corpus::default();
            for (_, tokens) in &tokens {
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
