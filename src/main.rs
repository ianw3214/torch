slint::include_modules!();

use std::rc::Rc;

use slint::{Model, ModelRc, VecModel};

#[derive(Clone)]
struct TreeNode {
    item : FolderItem,
    children : Vec<TreeNode>
}

fn flatten_visible_tree(nodes : &[TreeNode], depth : i32) -> Vec<FolderItem> {
    let mut result = Vec::new();

    for node in nodes {
        let mut item = node.item.clone();
        item.depth = depth;
        result.push(item.clone());

        if item.is_folder && item.is_expanded {
            result.extend(flatten_visible_tree(&node.children, depth + 1));
        }
    }

    result
}

fn build_tree(path : &std::path::Path) -> Vec<TreeNode> {
    let mut result = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
        let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let is_folder = path.is_dir();
            let name : slint::SharedString = entry.file_name().into_string().unwrap().into();
            
            let children = if is_folder {
                build_tree(&path)
            }
            else
            {
                vec![]
            };

            result.push(TreeNode {
                item : FolderItem { 
                    depth: 0, 
                    is_expanded: false, 
                    is_folder: is_folder, 
                    name: name,
                    full_path: path.into_os_string().into_string().unwrap().into() },
                children
            });
        }
    }

    result
}

fn toggle_folder_expansion(nodes: &mut [TreeNode], name: &str) -> bool {
    for node in nodes {
        if node.item.name == name && node.item.is_folder {
            node.item.is_expanded = !node.item.is_expanded;
            return true;
        }

        if toggle_folder_expansion(&mut node.children, name) {
            return true;
        }
    }
    false
}

fn is_text_file(path: &std::path::Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("txt" | "json" | "md" | "toml")
    )
}

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;
    let main_window_weak = main_window.as_weak();

    // TODO: Replace path w/ project path
    let path = std::path::Path::new("./");
    // let items = build_folder_items(path, 0);
    let mut tree = build_tree(path);
    let model_data = flatten_visible_tree(&tree, 0);

    let vec_model = Rc::new(VecModel::from(model_data));
    let model_rc = ModelRc::from(vec_model.clone());
    main_window.set_folders(model_rc);

    main_window.set_selected_index(-1);

    main_window.on_folder_clicked(move |index| {
        // Selection logic
        main_window_weak.unwrap().set_selected_index(index);
        
        // Expand/collapse logic
        let vec_model = vec_model.clone();
        let row = vec_model.row_data(index as usize);
        if let Some(item) = row {
            // Set preview info
            main_window_weak.unwrap().set_preview_name(item.name.clone());
            main_window_weak.unwrap().set_preview_path(item.full_path.clone());
            main_window_weak.unwrap().set_preview_type(if item.is_folder { "Folder".into() } else { "File".into() });
            if !item.is_folder && is_text_file(std::path::Path::new(item.full_path.as_str()))
            {
                match std::fs::read_to_string(item.full_path.as_str())
                {
                    Ok(contents) => {
                        let truncated = if contents.len() > 200 {
                            format!("{}\n\n[... truncated]", &contents[..200])
                        }
                        else
                        {
                            contents
                        };
                        main_window_weak.unwrap().set_preview_contents(truncated.into());
                    }
                    Err(err) => {
                        main_window_weak.unwrap().set_preview_contents(format!("Failed to read file: {}", err).into());
                    }
                }
            }
            else
            {
                main_window_weak.unwrap().set_preview_contents("".into());
            }

            // Toggle expanded flag in full tree
            if toggle_folder_expansion(&mut tree, &item.name) {
                let updated = flatten_visible_tree(&tree, 0);
                vec_model.set_vec(updated);
            }
        }
    });

    main_window.run()
}
