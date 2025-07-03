mod meteor;
mod stemmer;
mod synsets;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use serde_jsonlines::{json_lines, write_json_lines};
use std::io::Result;
use synsets::Synsets;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Document {
    pub repository_id: u32,
    pub content: String,
}

fn main() -> Result<()> {
    let mut synsets = Synsets::new();
    let mut stemmer = stemmer::Stemmer::new();

    let dataset = json_lines("/path/to/file.jsonl")?.collect::<Result<Vec<Document>>>()?;
    let count = dataset.len() as u64;
    let multi = MultiProgress::new();
    let pb1 = multi.add(ProgressBar::new(count));
    let pb2 = multi.add(ProgressBar::new(count));

    for a in &dataset {
        for b in &dataset {
            meteor::meteor_score(
                &a.content.split_whitespace().collect(),
                &b.content.split_whitespace().collect(),
                0.9,
                3.0,
                0.5,
                &mut synsets,
            );
            pb2.inc(1);
        }
        pb2.reset();
        pb1.inc(1);
    }

    Ok(())
}
