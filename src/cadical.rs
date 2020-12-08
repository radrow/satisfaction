use std::process::Command;

let output = if cfg!(target_os = "windows") {
    Command::new("cmd")
        .args(&["/C", "echo hello"])
        .output()
        .expect("failed to execute process")
} else {
    Command::new("sh")
        .arg("-c")
        .arg("echo hello")
        .output()
        .expect("failed to execute process")
};

let hello = output.stdout;


pub fn call_cadical(input : String) {
    let mut p = Command::new("cadical")
        .spawn().unwrap();
    p.stdin().get_mut_ref().write_str(input);
    p.wait();
}
