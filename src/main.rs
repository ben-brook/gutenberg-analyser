use reqwest::{blocking::Client, StatusCode};
use std::{collections::HashMap, io, sync::mpsc, thread};

const WORDS_PER_THREAD: usize = 10_000;
const FREQUENTS_SHOWN: usize = 30;

fn main() {
    let mut buf = String::new();
    let client = Client::new();

    println!("Welcome to my Project Gutenberg book analyser! Please input a book URL...");
    let txt = loop {
        io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        let given_id = buf.trim().trim_start_matches(|chr: char| !chr.is_numeric());
        if let Ok(id) = given_id.parse::<u32>() {
            if id > 0 {
                let res = client
                    .get(format!(
                        "https://www.gutenberg.org/cache/epub/{id}/pg{id}.txt"
                    ))
                    .send()
                    .unwrap(); // Safe to unwrap since URL will be parseable
                if let (StatusCode::OK, Ok(txt)) = (res.status(), res.text()) {
                    break txt;
                }
            }
        }
        println!("Invalid book URL. Try again...");
        buf.clear();
    };

    println!("Analysing text...");

    let mut words_iter = txt
        .split_whitespace()
        .map(|x| {
            x.trim_matches(|chr: char| !chr.is_alphanumeric())
                .to_lowercase()
        })
        .filter(|s| !s.is_empty());

    let (px, rx) = mpsc::channel();
    let mut is_unfinished = true;
    while is_unfinished {
        let mut words = Vec::with_capacity(WORDS_PER_THREAD);
        for _ in 0..WORDS_PER_THREAD {
            if let Some(word) = words_iter.next() {
                words.push(word);
            } else {
                is_unfinished = false;
                break; // Reached end of text
            }
        }

        let pxc = px.clone();
        thread::spawn(move || {
            let mut frequency_by_word = HashMap::new();
            for word in words {
                *frequency_by_word.entry(word).or_insert(0) += 1;
            }
            pxc.send(frequency_by_word).unwrap();
        });
    }
    drop(px);

    let mut frequency_by_word: Option<HashMap<String, u16>> = None;
    for sub_map in rx {
        if let Some(map) = &mut frequency_by_word {
            for (word, freq) in sub_map {
                *map.entry(word).or_insert(0) += freq;
            }
        } else {
            frequency_by_word = Some(sub_map);
        }
    }

    let mut most_frequent: [(String, u16); FREQUENTS_SHOWN] = Default::default();
    // We can unwrap frequency_by_word because there will always be at least one
    // thread spawned, since the while loop always runs at least once.
    for (word, freq) in frequency_by_word.unwrap() {
        let mut idx_to_push = None;
        for (idx, (_, ref_freq)) in most_frequent.iter().enumerate() {
            if freq > *ref_freq {
                idx_to_push = Some(idx);
                break;
            }
        }
        if let Some(idx) = idx_to_push {
            most_frequent[idx..].rotate_right(1);
            most_frequent[idx] = (word, freq);
        }
    }

    println!("The {FREQUENTS_SHOWN} most frequent words are:");
    for (idx, (word, freq)) in most_frequent.into_iter().enumerate() {
        println!("{}. {word}: {freq}", idx + 1);
    }
}
