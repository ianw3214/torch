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
                item : FolderItem { depth: 0, is_expanded: false, is_folder: is_folder, name: name },
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

// fn build_folder_items(path : &std::path::Path, depth : i32) -> Vec<FolderItem> {
//     let mut items = Vec::new();

//     if let Ok(entries) = std::fs::read_dir(path) {
//         let mut entries = entries.filter_map(Result::ok).collect::<Vec<_>>();
//         entries.sort_by_key(|e| e.file_name());
//         for entry in entries {
//             let path = entry.path();
//             let is_folder = path.is_dir();
//             items.push(FolderItem {
//                 // TODO: using to_string_lossy might result in less errors
//                 name: entry.file_name().into_string().unwrap().into(),
//                 depth,
//                 is_folder,
//                 is_expanded : false
//             });
            
//             if is_folder {
//                 items.extend(build_folder_items(&path, depth + 1));
//             }
//         }
//     }

//     items
// }

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    // TODO: Replace path w/ project path
    let path = std::path::Path::new("./");
    // let items = build_folder_items(path, 0);
    let mut tree = build_tree(path);
    let model_data = flatten_visible_tree(&tree, 0);

    // let model = ModelRc::new(VecModel::from(model_data));
    let vec_model = Rc::new(VecModel::from(model_data));
    let model_rc = ModelRc::from(vec_model.clone());
    main_window.set_folders(model_rc);

    main_window.on_folder_clicked(move |index| {
        let vec_model = vec_model.clone();
        let row = vec_model.row_data(index as usize);
        if let Some(item) = row {
            // Toggle expanded flag in full tree
            if toggle_folder_expansion(&mut tree, &item.name) {
                let updated = flatten_visible_tree(&tree, 0);
                vec_model.set_vec(updated);
            }
        }
    });

    main_window.run()
}
