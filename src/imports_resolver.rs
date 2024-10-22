use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::Path;

pub fn resolve_imports(module: &str, _module_exports: &Vec<String>, path: &Path) {
    let files = collect_files(&path);

    for (file_name, file_path) in files {
        let fp = Path::new(&file_path);
        match extract_imports(&fp, &module) {
            Some(component) => {
                println!("{:?} file : {}", component, file_path);
                let mut file_to_refactor =
                    OpenOptions::new()
                        .read(true)
                        .open(&file_path)
                        .expect(&String::from(format!(
                            "Failed to open {} Dir: {}",
                            &file_name, &file_path
                        )));
                let mut content = String::new();
                let _ = file_to_refactor.read_to_string(&mut content).unwrap();
                for cmp in component {
                    content = refactor(&mut content, &cmp);
                }
                let mut file_to_refactor = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&file_path)
                    .expect(&String::from(format!(
                        "Failed to open {} Dir: {}",
                        &file_name, &file_path
                    )));
                let _ = file_to_refactor.write_all(content.as_bytes());
                println!("{}", file_path)
            }
            None => (),
        }
    }
}
// Recursive collect files in the dir and subdir
fn collect_files(dir: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();

            // exlude `node_modules` directory
            if path.is_dir()
                && path
                    .file_name()
                    .map_or(false, |name| name == "node_modules")
            {
                continue;
            }

            if path.is_dir() {
                // Recurse
                files.extend(collect_files(&path));
            } else if let Some(extension) = path.extension() {
                // Check if the file ends with .js or .ts
                if extension == "js"
                    || extension == "ts"
                    || extension == "jsx"
                    || extension == "tsx"
                {
                    if let Some(file_name) = path.file_name() {
                        files.push((
                            file_name.to_string_lossy().to_string(),
                            path.to_string_lossy().to_string(),
                        ));
                    }
                }
            }
        }
    }

    files
}

fn extract_imports<'a>(path: &'a Path, module_name: &'a str) -> Option<Vec<String>> {
    let content = fs::read_to_string(path).unwrap();
    let pattern = format!(
        r#"\bimport\s*\{{*(\s*([^}}]+)\s*|\w+)\}}*\s*from\s*\"{}\""#,
        regex::escape(module_name)
    );
    let re = regex::Regex::new(&pattern).unwrap();
    let mut exports = None;
    if let Some(captures) = re.captures(&content) {
        match captures.get(1) {
            Some(r) => {
                exports = Some(
                    r.as_str()
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
                        .collect::<Vec<String>>(),
                );

                return exports;
            }
            None => return None,
        }
    }
    exports
}

fn refactor(content: &mut String, component: &str) -> String {
    // html
    let target_tag = component; 
    let replacement_tag = "div";

    let target_opening_tag = format!("<{}", target_tag);
    let target_closing_tag = format!("</{}>", target_tag);

    let modified_content = content
        .replace(
            &target_opening_tag,
            &format!(
                "\r\n{{/* {} Refactored */}}\r\n<{}",
                component, replacement_tag
            ),
        )
        .replace(&target_closing_tag, &format!("</{}>", replacement_tag));

    // hooks
    /*
    todo making a fn that detect callbacks inside hooks
    example :

    const {a , b} = useHook({
        ctx: (c) => {
        //some logic
        },
    });

     */
    let pattern = format!(
        r"(\w+)\s*(\w+)\s*=\s*({})\s*\((.*?)\)\s*",
        regex::escape(component)
    );
    let re = regex::Regex::new(&pattern).unwrap();
    println!("{:?}", re.captures(&modified_content));
    // Replace matches with commented-out versions
    let result = re
        .replace_all(&modified_content, |caps: &regex::Captures| {
            let matched = &caps[0]; // The full match including `useButton`
            format!("/*\n{} Refactored\n{}\n*/", component, matched) // Comment out the matched declaration
        })
        .to_string();

    result
}
