use std::{env, fs};
// use std::path::{Path, PathBuf};
use std::path::PathBuf;
use std::time::Instant;
use std::sync::Arc;
use std::cmp::max;
use rayon::prelude::*;

fn main() {
    let start = Instant::now();

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 && args.len() != 4 {
        panic!("Usage: {} <directory> <query> <(optional) search type>", args[0]);
    }
    let directory = Arc::new(PathBuf::from(&args[1]));
    let query = Arc::new(args[2].to_lowercase());
    let search_type = if args.len() == 4 {
        match args[3].as_str() {
            "substring" | "exact" | "content" | "fuzzy" => Arc::new(args[3].clone()),
            _ => panic!("Invalid search type: must be 'substring', 'exact', 'content', or 'fuzzy'"),
        }
    } else {
        Arc::new("substring".to_string())
    };

    rayon::scope(|s| {
        s.spawn(move |_| {
            spawn_worker(directory, query, search_type);
        });
    });

    let duration = start.elapsed();
    println!("\nFinished: {:?}", duration);
}

fn spawn_worker(directory: Arc<PathBuf>, query: Arc<String>, search_type: Arc<String>) {
    let entries_result = fs::read_dir(&*directory);
    let entries: Vec<_> = match entries_result {
        Ok(entries) => entries.collect(),
        Err(e) => {
            eprintln!("\n----------------------------------\nError reading directory:\n{} \n{} \n----------------------------------", directory.display(), e);
            return;
        }
    };

    let chunk_size = max(1, entries.len() / rayon::current_num_threads().max(1));

    entries.par_chunks(chunk_size).for_each(|chunk| {
        for entry in chunk {
            let entry = entry.as_ref().unwrap();
            let path = entry.path();

            if path.is_dir() {
                let directory = Arc::new(path);
                let query_clone = Arc::clone(&query);
                let search_type_clone = Arc::clone(&search_type);
                rayon::scope(move |s| {
                    s.spawn(move |_| {
                        spawn_worker(directory, query_clone, search_type_clone);
                    });
                });

            } else {
                let path = Arc::new(path);
                let query_clone = Arc::clone(&query);
                let search_type_clone = Arc::clone(&search_type);
                rayon::scope(move |s| {
                    s.spawn(move |_| {
                        spawn_evaluator(path, query_clone, search_type_clone);
                    });
                });
            }
        }
    });
}

fn spawn_evaluator(path: Arc<PathBuf>, query: Arc<String>, search_type: Arc<String>) {
    let search_type = &*search_type;

    if search_type == "substring" {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_lowercase();
        if file_name.contains(&*query) {
            display_result(path.display().to_string());
        }

    } else if search_type == "exact" {
        let file_name = path.file_name().unwrap().to_str().unwrap().to_lowercase();
        if file_name == *query {
            display_result(path.display().to_string());
        }

    } else {
        panic!("Invalid search type: {}", search_type);
    }
}

fn display_result(path: String) {
    let normalized_path = path.replace(" ", "\\ ");
    println!("{}", normalized_path);
}
