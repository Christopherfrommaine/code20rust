mod wlrendering;
mod fa;
mod cellularautomation;
mod fmt_fa;
mod testing;
mod solver;
mod minimization;
mod bitset;


use crate::wlrendering::{render_prepare, render_postpare};

fn main() {
    env_logger::init();

    render_prepare();
    testing::main();
    render_postpare();
}