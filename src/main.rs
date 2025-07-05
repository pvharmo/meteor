mod meteor;
mod stemmer;
mod synsets;
use dotenv::dotenv;
use indicatif::ProgressIterator;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_jsonlines::json_lines;
use std::fmt::Write;
use std::io::Result;
use synsets::Synsets;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Info {
    pub stats: Stats,
}

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Stats {
    pub NumOfJobs: u32,
    pub TotalNumOfSteps: u32,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Document {
    pub repository_id: u32,
    pub content: String,
    pub info: Info,
}

fn main() -> Result<()> {
    dotenv().ok();
    let mut synsets = Synsets::new();
    let mut stemmer = stemmer::Stemmer::new();

    let dataset = json_lines(std::env::var("DATASET_PATH").expect("DATASET_PATH must be set."))?
        .collect::<Result<Vec<Document>>>()?;
    println!("Loaded dataset with {} rows", dataset.len());

    let mut categorized: Vec<Vec<(u64, Vec<&str>)>> = vec![vec![], vec![], vec![], vec![]];

    for doc in dataset.iter().progress() {
        let num_of_jobs = doc.info.stats.NumOfJobs;
        let num_of_steps = doc.info.stats.TotalNumOfSteps;
        let content = doc.content.split_whitespace().collect();
        meteor::init_cache(&content, &mut synsets, &mut stemmer).unwrap();
        if num_of_jobs > 1 {
            categorized[0].push((doc.repository_id as u64, content));
        } else if num_of_steps > 5 {
            categorized[1].push((doc.repository_id as u64, content));
        } else if num_of_steps > 3 {
            categorized[2].push((doc.repository_id as u64, content));
        } else {
            categorized[3].push((doc.repository_id as u64, content));
        }
    }

    println!(
        "category splits: {}, {}, {}, {}",
        categorized[0].len(),
        categorized[1].len(),
        categorized[2].len(),
        categorized[3].len()
    );

    for (dataset_index, preprocessed_dataset) in categorized.iter().enumerate() {
        let preprocessed_dataset_2 = preprocessed_dataset.clone();
        let count = preprocessed_dataset.len();

        let pbi = ProgressBar::new(count as u64);

        let style = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {eta} {pos:>7}/{len:7}",
        )
        .unwrap()
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#>-");

        pbi.set_style(style);

        println!("Preprocessing done, starting scoring {} elements", count);
        let score_matrix: Vec<Vec<u8>> = preprocessed_dataset
            .into_par_iter()
            .enumerate()
            .map(|(i, a)| {
                let mut score_vector = vec![0 as u8; count];
                pbi.inc(1);
                for (j, b) in preprocessed_dataset_2.iter().enumerate() {
                    if j <= i {
                        continue;
                    }
                    let score = meteor::meteor_score(&a.1, &b.1, 0.9, 3.0, 0.5, &synsets, &stemmer);
                    score_vector[j] = (score * 100.0).round() as u8;
                }
                score_vector
            })
            .collect();

        std::fs::write(
            format!("score_matrix-{}.bin", dataset_index),
            score_matrix.into_iter().flatten().collect::<Vec<u8>>(),
        )
        .unwrap();
    }

    Ok(())
}
