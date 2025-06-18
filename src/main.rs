slint::include_modules!();

use slint::{ModelRc, VecModel};

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    let path = "./";
    let folder_names = std::fs::read_dir(path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect::<Vec<_>>();

    // Need to convert rust strings to slint usable strings
    let folder_names : Vec<slint::SharedString> = folder_names.into_iter().map(Into::into).collect();
    // A modelRc is a reference counted wrapper to a model that allows rust to provide dynamic lists to slint
    let model = ModelRc::new(VecModel::from(folder_names));
    main_window.set_folders(model);

    main_window.run()
}
