use std::process::Command;

fn main() {
    let inf = Command::new("ln")
        .arg("-f")
        .arg("/home/twhicer/code/terl/target/debug/terl")
        .arg("/bin/terl")
        .output()
        .expect("Sudo");
}
