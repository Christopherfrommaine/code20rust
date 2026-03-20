use std::{fs::{File, OpenOptions}, io::Write, process::{Command, Stdio}};

const PATH: &'static str = "./renders/generator.wls";

pub fn render_prepare() {
    Command::new("rm")
        .arg("-r")
        .arg("./renders/")
        .status()
        .expect("");

    Command::new("mkdir")
        .arg("renders")
        .status()
        .expect("");
    
    let mut file = File::create(PATH).expect("Failed to create file");
    file.write_all("#!/usr/bin/env wolframscript\n\n".as_bytes()).expect("Failed to write to file");
    
    Command::new("chmod")
        .arg("+x")
        .arg(PATH)
        .status()
        .expect("couldnt set permissions");
}

pub fn render(cmd: String, path: &str) {
    let input = format!("Export[{:?}, Rasterize[{cmd}]]; \n", path.to_string() + ".png").replace(" ", "");

    let mut file = OpenOptions::new()
    .append(true)
    .create(false)
    .open(PATH)
    .expect("failed to open file");

    file.write_all(input.as_bytes()).expect("couldnt write to file");
}

pub fn render_postpare() {
    Command::new(PATH)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .expect("couldnt run file");
}