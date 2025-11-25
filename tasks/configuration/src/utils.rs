use std::path::PathBuf;
use std::process::Command;

/// Get the workspace root directory by executing `cargo locate-project --workspace`
///
/// # Panics
/// Panics if the cargo command fails or if the output cannot be parsed
pub fn get_workspace_root() -> PathBuf {
    // Execute the `cargo locate-project --workspace` command
    let output = Command::new("cargo")
        .arg("locate-project")
        .arg("--workspace")
        .output()
        .expect("Failed to execute cargo command");

    // Convert the command output to a string
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Extract the path to the workspace Cargo.toml from the JSON output
    let json: serde_json::Value =
        serde_json::from_str(&output_str).expect("Failed to parse cargo output as JSON");
    let cargo_toml_path = json["root"]
        .as_str()
        .expect("Failed to get 'root' field from JSON");

    // Get the workspace root directory by stripping the Cargo.toml filename
    std::path::Path::new(cargo_toml_path)
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf()
}
