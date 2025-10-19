mod test;
mod tokenizers;
mod tokens_scorer;

use crate::tokens_scorer::TokensScorer;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ProjectLabeler {
    file_tokens_scorer: TokensScorer,
    path_tokens_scorer: TokensScorer,
}

impl ProjectLabeler {
    pub fn new(project_path: String, project_files: Vec<&str>) -> Self {
        let files = project_files
            .into_iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();

        let mut file_tokens_scorer = TokensScorer::new(move |path| tokenizers::file(path));
        file_tokens_scorer.ingest(&files);

        let mut path_tokens_scorer =
            TokensScorer::new(move |path| Ok(tokenizers::path(path, &project_path)));
        path_tokens_scorer.ingest(&files);

        Self { file_tokens_scorer, path_tokens_scorer }
    }

    pub fn label_files(
        &self,
        selected_files: Vec<PathBuf>,
    ) -> Result<HashMap<String, f64>, std::io::Error> {
        let file_scores = {
            measure_time::info_time!("Processing files for TF-IDF scores");
            self.file_tokens_scorer.run(&selected_files)?
        };

        let path_scores = {
            measure_time::info_time!("Processing file paths for TF-IDF scores");
            self.path_tokens_scorer.run(&selected_files)?
        };

        let final_scores = {
            measure_time::info_time!("Calculating final score");
            let mut combined_scores: HashMap<String, f64> = HashMap::new();
            for (token, score) in file_scores.iter().chain(path_scores.iter()) {
                *combined_scores.entry(token.clone()).or_insert(0.0) += score.clone() as f64;
            }
            combined_scores
        };

        Ok(final_scores)
    }
}
