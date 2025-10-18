mod tokenizers;
mod tokens_scorer;
mod test;

use crate::tokens_scorer::TokensScorer;
use std::collections::HashMap;
use std::path::Path;

pub fn label_files(
    project_files: Vec<&str>,
    selected_files: Vec<&str>,
    project_path: &str,
) -> Result<HashMap<String, f64>, std::io::Error> {
    let files = project_files.into_iter().map(Path::new).collect::<Vec<_>>();
    let selected_files = selected_files
        .into_iter()
        .map(Path::new)
        .collect::<Vec<_>>();

    let file_scores = {
        measure_time::info_time!("Processing files for TF-IDF scores");
        let tokens_scorer = TokensScorer::new(move |path| tokenizers::file(path));
        tokens_scorer.run(&files, &selected_files)?
    };

    let path_scores = {
        measure_time::info_time!("Processing file paths for TF-IDF scores");
        let project_path = project_path.to_string();
        let labeler = TokensScorer::new(move |path| Ok(tokenizers::path(path, &project_path)));
        labeler.run(&files, &selected_files)?
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
