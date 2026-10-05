fn main() {
    tauri_build::build();

    // A UI é embarcada no binário (custom-protocol, sem devUrl). Sem isto,
    // editar ui/ não recompila a crate e o binário serve HTML velho.
    // Lista arquivos (não o diretório: cargo só olha mtime de diretório,
    // que não muda ao editar conteúdo).
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
