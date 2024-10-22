use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
struct PackageJson {
    dependencies: Option<HashMap<String, String>>,
    dev_dependencies: Option<HashMap<String, String>>,
}

static NODE_INDEX: &[&str; 2] = &["index.mjs", "index.js"];

pub fn resolve_module<'a>(pack: &'a str, path: &'a Path) -> Result<Vec<String>, &'a str> {
    let file_content = fs::read_to_string("package.json").expect("Unable to read package.json");

    // Parse the JSON into the PackageJson struct
    let package_json: PackageJson =
        serde_json::from_str(&file_content).expect("Invalid JSON format");
    let binding = path.join("node_modules");
    let node_modules = Path::new(&binding);

    if !node_modules.exists() {
        Err("node_modules doesn't exist in the current dir!\nTry \t'npm i'\t befor using refactor")
    } else {
        let mut exports = vec![];
        if let Some(dependencies) = package_json.dependencies {
            if !is_indeps(&pack, &dependencies) {
                return Err("Module is not present in the package.json");
            } else {
                let node_modules = &node_modules.join(Path::new(&pack.replace("/", "\\")));
                let files_to_find = NODE_INDEX; // Add your target files here
                let files = search_files(node_modules, files_to_find);
                for file_name in files.to_vec() {
                    let path = Path::new(&file_name);
                    exports = read_and_extract_exports(&path).unwrap();
                }
            }
        }

        if let Some(dev_dependencies) = package_json.dev_dependencies {
            if !is_indeps(&pack, &dev_dependencies) {
                return Err("Module is not present in the package.json");
            } else {
                let node_modules = &node_modules.join(Path::new(&pack.replace("/", "\\")));
                let files_to_find = NODE_INDEX; // Add your target files here
                let files = search_files(node_modules, files_to_find);
                for file_name in files.to_vec() {
                    let path = Path::new(&file_name);
                    exports = read_and_extract_exports(&path).unwrap();
                }
            }
        }
        Ok(exports)
    }
}

fn is_indeps(package: &str, deps: &HashMap<String, String>) -> bool {
    let mut result = false;
    for (name, _) in deps {
        if name == package {
            result = true;
        }
    }
    result
}

fn search_files<P: AsRef<Path>>(dir: P, target_files: &[&str]) -> Vec<PathBuf> {
    let mut found_files = Vec::new();
    let dir = dir.as_ref();

    // Read the directory
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            // Check if the entry is a directory or a file
            if path.is_dir() {
                // Recursively search in subdirectories
                found_files.extend(search_files(&path, target_files));
            } else if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                // Check if the file name matches any target file names
                if target_files.contains(&file_name) {
                    found_files.push(path);
                }
            }
        }
    }

    found_files
}

fn read_and_extract_exports(path: &Path) -> Result<Vec<String>, std::io::Error> {
    // Read the file content
    let content = fs::read_to_string(path)?;

    // Determine the file extension
    let extension = path.extension().and_then(|s| s.to_str());

    // Extract exports based on the file type
    let mut exports = Vec::new();
    match extension {
        Some("mjs") => {
            for expo in extract_exports(&content) {
                // Check if value is not already in vec1
                if !exports.contains(&expo) {
                    exports.push(expo); // Push value from vec2 into vec1
                }
            }
        }
        Some("js") => {
            for expo in extract_exports(&content) {
                // Check if value is not already in vec1
                if !exports.contains(&expo) {
                    exports.push(expo); // Push value from vec2 into vec1
                }
            }
        }
        _ => {}
    }
    Ok(exports)
}

fn extract_exports(content: &str) -> Vec<String> {
    let re = regex::Regex::new(r"export\s*\{([^}]+)\}").unwrap();
    let mut exports = Vec::new();
    if let Some(captures) = re.captures(content) {
        match captures.get(1) {
            Some(r) => {
                exports = r
                    .as_str()
                    .split(',')
                    .filter_map(|s| {
                        let trimmed = s.trim();
                        // Capture the identifier after 'as' if it exists, or just the identifier
                        if let Some(as_index) = trimmed.find(" as ") {
                            Some(trimmed[as_index + 4..].to_string())
                        } else {
                            Some(trimmed.to_string())
                        }
                    })
                    .collect();
            }
            None => {}
        }
    }
    exports
}
