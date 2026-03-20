mod fsm;
mod vis;

use fsm::{finitestatemachine, vec_to_u64};

fn main() {
    let p = 22;

    let states = finitestatemachine(p, -1, 100);
    println!("States: {states:?}");

    let nstates: Vec<u64> = states.iter().map(|s| vec_to_u64(s.to_vec())).collect();
    println!("NStates: {nstates:?}");
}
