pub mod int;
pub mod handle_solution;
pub mod solver;

use crate::solver::*;
use crate::handle_solution::{clear_renders, clear_output_file, read_starting_index};

fn main() {
    // Pickup from last computation
    let starting_index = read_starting_index();

    // Initialization
    clear_renders();
    // clear_output_file();

    // Fix stack overflow during deep recursion with large integers
    rayon::ThreadPoolBuilder::new()
        .stack_size(1024 * 1024 * 1024) // stack size in bytes
        .build_global()
        .unwrap();

    // Solve for all new periods
    for p in starting_index.. {
        solve(p, 0);
    }
}
