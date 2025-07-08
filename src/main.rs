mod meteor;
mod stemmer;
mod synsets;
use indicatif::ProgressIterator;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use itertools::Itertools;
use std::cmp::min;

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
    pub id: u32,
    pub content: String,
    pub info: Info,
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut synsets = Synsets::new();
    let mut stemmer = stemmer::Stemmer::new();

    let dataset = json_lines(&args[1])
        .expect("You must provide a valid path to a JSON Lines file as first argument")
        .collect::<Result<Vec<Document>>>()
        .expect("Invalid JSON Lines file provided");
    println!("Loaded dataset with {} rows", dataset.len());

    let mut categorized: Vec<Vec<(u64, Vec<&str>)>> = vec![vec![]; 100];

    for doc in dataset.iter().progress() {
        let num_of_jobs = min(doc.info.stats.NumOfJobs, 10) as i64 - 1;
        let num_of_steps = min(doc.info.stats.TotalNumOfSteps, 10) as i64 - 1;
        let content = doc.content.split_whitespace().collect();

        meteor::init_cache(&content, &mut synsets, &mut stemmer).unwrap();

        if num_of_steps >= 0 && num_of_jobs >= 0 {
            categorized[(num_of_jobs * 10 + num_of_steps) as usize].push((doc.id as u64, content))
        }
    }

    println!("Categories counts: ");
    for (i, category) in categorized.iter().enumerate() {
        if i % 10 == 0 && i != 0 {
            println!();
        }
        print!("{}, ", category.len());
    }
    println!();

    calculate_score(&categorized, &synsets, &stemmer).unwrap();

    filter_dataset(categorized, &args[2]).unwrap();

    Ok(())
}

fn calculate_score(
    categorized_dataset: &Vec<Vec<(u64, Vec<&str>)>>,
    synsets: &Synsets,
    stemmer: &stemmer::Stemmer,
) -> Result<()> {
    for (dataset_index, preprocessed_dataset) in categorized_dataset.iter().enumerate() {
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
            // .iter()
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

fn filter_dataset(
    categorized_dataset: Vec<Vec<(u64, Vec<&str>)>>,
    file_path: &String,
) -> Result<()> {
    let mut indices_to_remove: Vec<u64> = Vec::with_capacity(10000);
    for (dataset_index, preprocessed_dataset) in categorized_dataset.iter().enumerate() {
        let meteor_scores = std::fs::read(format!("score_matrix-{}.bin", dataset_index)).unwrap();
        // for scores_chunks in &meteor_scores.iter().chunks(preprocessed_dataset.len()) {
        for (j, score) in meteor_scores.iter().enumerate() {
            // for (j, score) in scores_chunks.enumerate() {
            if *score > 95 {
                indices_to_remove.push(preprocessed_dataset[j % preprocessed_dataset.len()].0)
            }
            // }
        }
    }

    let indices_to_remove_string = indices_to_remove
        .iter()
        .unique()
        .map(|score| score.to_string())
        .join("\n");

    std::fs::write(file_path, format!("id\n{}", indices_to_remove_string)).unwrap();
    Ok(())
}
