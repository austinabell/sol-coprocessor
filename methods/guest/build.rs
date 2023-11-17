use std::process::Command;

fn main() {
    // TODO think more around where the build file should be.
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    // let out_dir = std::path::Path::new(&out_dir);
    // let cache_dir = out_dir.join(".cache");

    let mut forge_build = Command::new("forge")
        .arg("build")
        // .arg("--cache-path")
        // .arg(cache_dir)
        // .arg("--out")
        // .arg(out_dir)
        .spawn()
        .expect("failed to start `forge build`");

    let status = forge_build.wait().expect("failed to run `forge build`");
    if !status.success() {
        panic!("`forge build` exited with failed status: {}", status);
    }
}
