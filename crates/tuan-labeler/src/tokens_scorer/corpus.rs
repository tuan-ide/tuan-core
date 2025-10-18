use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Corpus {
    pub n_docs: usize,
    pub df: HashMap<String, usize>,
}

impl Corpus {
    pub fn ingest_document(&mut self, tokens: &[String]) {
        self.n_docs += 1;
        let mut seen: HashSet<&str> = HashSet::new();
        for t in tokens {
            if seen.insert(t.as_str()) {
                *self.df.entry(t.clone()).or_insert(0) += 1;
            }
        }
    }

    pub fn tfidf(&self, tokens: &[String]) -> Vec<(String, f32)> {
        let mut tf: HashMap<&str, usize> = HashMap::new();
        for t in tokens {
            *tf.entry(t.as_str()).or_insert(0) += 1;
        }
        let len = tokens.len().max(1) as f32;

        let mut scored: Vec<(String, f32)> = tf
            .into_iter()
            .map(|(tok, f)| {
                let tf_norm = f as f32 / len;
                let df = *self.df.get(tok).unwrap_or(&0) as f32;
                let idf = ((self.n_docs as f32 + 1.0) / (df + 1.0)).ln() + 1.0;
                (tok.to_string(), tf_norm * idf)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored
    }
}
