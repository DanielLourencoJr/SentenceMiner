fn main() {
    tauri_build::build();

    // The UI is embedded into the binary (custom-protocol, no devUrl). Without this,
    // editing ui/ would not recompile the crate and the binary would serve stale HTML.
    // List files (not the directory: cargo only looks at directory mtime,
    // which does not change when contents are edited).
    watch_dir_all_files("../ui");
}

fn watch_dir_all_files(dir: &str) {
    let mut stack = vec![std::path::PathBuf::from(dir)];
    while let Some(path) = stack.pop() {
        let entries = match std::fs::read_dir(&path) {
            Ok(entries) => entries,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
