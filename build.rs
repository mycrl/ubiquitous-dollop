use std::path::Path;
use std::process::*;
use std::{env, fs};

fn join(root: &str, next: &str) -> String {
    Path::new(root).join(next).to_str().unwrap().to_string()
}

#[cfg(target_os = "windows")]
fn exec(cmd: &str, work_dir: &str) -> Result<ExitStatus, std::io::Error> {
    Command::new("powershell")
        .args(["-command", cmd])
        .current_dir(work_dir)
        .status()
}

fn main() {
    let cef_version = "cef_binary_116.0.22+g480de66+chromium-116.0.5845.188";
    let is_debug = env::var("DEBUG")
        .map(|label| label == "true")
        .unwrap_or(true);
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = join(
        &join(&project_dir, "./target"),
        if is_debug { "./debug" } else { "./release" },
    );

    let temp = env::var("TEMP").unwrap();
    let temp_cef = join(&temp, cef_version);

    if fs::metadata(&join(&out_dir, "./locales")).is_err() {
        exec(&format!("cp -r {}/* ./", &temp_cef), &out_dir).unwrap();
    }
}
