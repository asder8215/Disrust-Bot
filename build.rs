use std::process::Command;
use std::env;

fn main() {
    // Check if 'nasm' is already installed (optional)
    let nasm_check = Command::new("which")
        .arg("nasm")
        .output();

    if let Ok(output) = nasm_check {
        if !output.stdout.is_empty() {
            println!("cargo:rerun-if-changed=build.rs"); // Trigger rebuild if necessary
            return;
        }
    }

    // Command to install nasm (works for Linux-based systems, e.g., Ubuntu)
    let install_nasm = Command::new("apt-get")
        .arg("install")
        .arg("-y")
        .arg("nasm")
        .output();

    if let Err(e) = install_nasm {
        eprintln!("Failed to install nasm: {}", e);
        std::process::exit(1);
    }

    println!("cargo:rerun-if-changed=build.rs"); // Trigger rebuild if necessary
}
