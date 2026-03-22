use crate::solver::code20;
use crate::int::*;
use std::{fs, path::Path, io::Write};

fn bits(n: Int, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| to_u8((n >> i) & one()))
        .rev()
        .collect()
}

pub fn log_to_file(string: String) {
    let filename = "output.txt";

    let mut file = match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    if std::io::Write::write_all(&mut file, string.as_bytes()).is_err() {
        eprintln!("Error writing log to file!");
        return;
    }
}

pub fn save_render_to_file(string: String, filename: &str) {
    
    let mut file = match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    if std::io::Write::write_all(&mut file, string.as_bytes()).is_err() {
        eprintln!("Error writing log to file!");
        return;
    }
}

fn log_solution_to_file(sol: Int, period: usize, shift: usize) {
    log_to_file(format!("n{sol} p{period} s{shift}\n"));
}

fn solution_to_array(sol: Int, period: usize, shift: usize) -> Vec<Vec<u8>> {
    let repititions = 10;
    let padding = 10 + shift * repititions;

    let w: usize = sol.ilog2() as usize + 2 * padding;

    let mut state = sol << padding;
    let mut arr = vec![bits(state, w)];
    for _ in 0..(repititions * period) {
        state = code20(state);
        arr.push(bits(state, w));

        if state == zero() {
            break;
        }
    }

    arr
}

fn array_plot(arr: Vec<Vec<u8>>) -> String {
    arr.into_iter().map(|row|
        row.into_iter().map(|e|
        
        if e == 0 {' '}
        else if e == 1 {'█'}
        else {'?'}

        ).collect::<String>()
    ).collect::<Vec<String>>().join("\n")
}

fn save_rendered_solution(sol: Int, period: usize, shift: usize) {
    let arr = solution_to_array(sol, period, shift);
    
    save_render_to_file(array_plot(arr), &format!("renders/solution_p{period}_s{shift}_n{sol}.txt"));
}

pub fn handle_found_solution(sol: Int, period: usize) {
    let shift = 0;
    println!("Found Solution: {sol}");

    log_solution_to_file(sol, period, shift);
    save_rendered_solution(sol, period, shift);
}

pub fn clear_renders() {
    let dir = Path::new("./renders");
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path);
                }
            }
        }
    } else {
        match fs::create_dir(dir) {
            Ok(_) => {}
            Err(e) => {eprintln!("Error creating render directory: {}", e)}
        }
    }
}

pub fn clear_output_file() {
    let _ = fs::File::create("output.txt").and_then(|mut f| f.write_all(b""));
}

pub fn read_starting_index() -> usize {
    std::fs::read_to_string("output.txt").unwrap_or(String::new())
    .lines().filter_map(|line|
        if line.starts_with("(p") {
            line[2..(line.len() - 6)].parse::<usize>().ok()
        } else {None}
    ).last().unwrap_or(1)
}