slint::include_modules!();

use slint::{ModelRc, VecModel};

fn build_folder_items(path : &std::path::Path, depth : i32) -> Vec<FolderItem> {
    let mut items = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let is_folder = path.is_dir();
            items.push(FolderItem {
                // TODO: using to_string_lossy might result in less errors
                name: entry.file_name().into_string().unwrap().into(),
                depth,
                is_folder,
                is_expanded : false
            });
            
            if is_folder {
                items.extend(build_folder_items(&path, depth + 1));
            }
        }
    }

    items
}

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    // TODO: Replace path w/ project path
    let path = std::path::Path::new("./");
    let items = build_folder_items(path, 0);
    let model = ModelRc::new(VecModel::from(items));
    main_window.set_folders(model);

    main_window.run()
}
