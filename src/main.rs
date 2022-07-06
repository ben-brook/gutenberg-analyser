use chrono::prelude::*;
use dashmap::DashMap;
use rayon::prelude::*;
use reqwest::{blocking::Client, StatusCode};
use std::io;

const FREQUENTS_SHOWN: usize = 30;

fn ask_txt() -> String {
    let mut buf = String::new();
    let client = Client::new();

    loop {
        io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        let given_id = buf.trim().trim_start_matches(|chr: char| !chr.is_numeric());
        if let Ok(id) = given_id.parse::<u32>() {
            if id > 0 {
                // Try first URL format
                let res = client
                    .get(format!(
                        "https://www.gutenberg.org/cache/epub/{id}/pg{id}.txt"
                    ))
                    .send()
                    .unwrap(); // Safe to unwrap since URL will be parseable
                if let (StatusCode::OK, Ok(txt)) = (res.status(), res.text()) {
                    break txt;
                }

                // Try second URL format
                let res = client
                    .get(format!("https://www.gutenberg.org/files/{id}/{id}-0.txt"))
                    .send()
                    .unwrap(); // Safe to unwrap since URL will be parseable
                if let (StatusCode::OK, Ok(txt)) = (res.status(), res.text()) {
                    break txt;
                }
            }
        }
        println!("Invalid book URL. Try again...");
        buf.clear();
    }
}

fn get_most_frequent(frequency_by_word: DashMap<String, u32>) -> [(String, u32); FREQUENTS_SHOWN] {
    const DEFAULT: (String, u32) = (String::new(), 0u32);
    let mut most_frequent = [DEFAULT; FREQUENTS_SHOWN];

    for (word, freq) in frequency_by_word {
        if let Some(idx) = most_frequent
            .iter()
            .position(|(_, ref_freq)| freq > *ref_freq)
        {
            most_frequent[idx..].rotate_right(1);
            most_frequent[idx] = (word, freq);
        }
    }

    most_frequent
}

fn main() {
    println!("Welcome to my Project Gutenberg book analyser! Please input a book URL...");
    let txt = ask_txt();
    println!("Analysing text...");
    let ts = Utc::now().timestamp_nanos();

    let frequency_by_word = DashMap::new();
    txt.par_split_whitespace()
        .map(|x| {
            x.trim_matches(|chr: char| !chr.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty())
        .for_each(|word| {
            *frequency_by_word.entry(word).or_insert(0) += 1;
        });

    let most_frequent = get_most_frequent(frequency_by_word);
    println!(
        "Found the {FREQUENTS_SHOWN} most frequent words in {} seconds:",
        (Utc::now().timestamp_nanos() - ts) as f32 / 1_000_000_000.
    );
    for (idx, (word, freq)) in most_frequent.into_iter().enumerate() {
        println!("{}. {word}: {freq}", idx + 1);
    }
}
