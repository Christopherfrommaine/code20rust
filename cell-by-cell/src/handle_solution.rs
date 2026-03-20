use crate::solver::code20;
use crate::int::*;
use std::{fs, path::Path, io::Write};

fn bits(n: Int, len: usize) -> Vec<u8> {
    (0..len)
        .map(|i| to_u8((n >> i) & one()))
        .rev()
        .collect()
}

fn log_solution_to_file(sol: Int, period: usize, shift: usize) {
    let filename = "output.txt";
    let string = format!("n{sol} p{period} s{shift}\n");

    let mut file = match std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    if std::io::Write::write_all(&mut file, string.as_bytes()).is_err() {
        return;
    }
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

#[allow(dead_code)]
pub fn print_plotted_solution(sol: Int, period: usize, shift: usize) {
    let arr = solution_to_array(sol, period, shift);

    use cgrustplot::{plots::array_plot::array_plot};
    array_plot(&arr)
        .set_axes(false)
        .set_title(&format!("p{period} s{shift} n{sol}"))
        .print();
}

fn save_rendered_solution(sol: Int, period: usize, shift: usize) {
    let arr = solution_to_array(sol, period, shift);

    let string = format!("p{period}_s{shift}_n{sol}");
    let string = if string.len() < 100 {string} else {string[..100].to_string()};


    use cgrustplot::{plots::array_plot::array_plot};
    array_plot(&arr)
        .set_axes(false)
        .set_title(&string)
        // .save(&format!("renders/solution_p{period}_s{shift}_n{sol}.txt"))
        .as_image()
        .save(&format!("renders/solution_{string}.png"));
}

pub fn handle_found_solution(sol: Int, period: usize, shift: usize) {
    
    println!("Found Solution: {sol}");

    log_solution_to_file(sol, period, shift);
    save_rendered_solution(sol, period, shift);
}

#[allow(unused_variables)]
pub fn handle_found_solution_poorly(sol: Int, period: usize, shift: usize) {
    
    println!("Found Solution: {sol}");
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
    }
}

pub fn clear_output_file() {
    let _ = fs::File::create("output.txt").and_then(|mut f| f.write_all(b""));
}
