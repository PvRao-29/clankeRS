//! Verify bundled node templates compile after placeholder substitution.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TEMPLATES: &[&str] = &[
    "basic_node",
    "perception_node",
    "ml_inference_node",
    "replay_test_node",
];

#[test]
fn templates_compile_after_scaffolding() {
    let workspace = workspace_root();
    let templates_root = workspace.join("templates");

    for name in TEMPLATES {
        let src = templates_root.join(name);
        if !src.join("Cargo.toml").exists() {
            continue;
        }
        let dest = workspace.join("target").join("template-check").join(name);
        let _ = fs::remove_dir_all(&dest);
        copy_dir(&src, &dest).expect("copy template");
        replace_in_tree(&dest, "{{PROJECT_NAME}}", &format!("tpl_{name}"));
        let clankers_path = relative_path(&dest, &workspace.join("crates/clankers"))
            .unwrap_or_else(|| PathBuf::from("../../../crates/clankers"));
        replace_in_tree(
            &dest,
            "{{CLANKERS_DEPENDENCY}}",
            &format!(r#"clankers = {{ path = "{}" }}"#, clankers_path.to_string_lossy()),
        );

        let status = Command::new("cargo")
            .arg("check")
            .arg("--manifest-path")
            .arg(dest.join("Cargo.toml"))
            .status()
            .expect("cargo check template");
        assert!(status.success(), "template {name} failed to compile");
    }
}

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().unwrap().parent().unwrap().to_path_buf()
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = to.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn replace_in_tree(root: &Path, needle: &str, replacement: &str) {
    fn walk(dir: &Path, needle: &str, replacement: &str) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(&path, needle, replacement);
            } else if let Ok(contents) = fs::read_to_string(&path) {
                if contents.contains(needle) {
                    fs::write(&path, contents.replace(needle, replacement)).unwrap();
                }
            }
        }
    }
    walk(root, needle, replacement);
}

fn relative_path(from: &Path, to: &Path) -> Option<PathBuf> {
    let from = from.components().collect::<Vec<_>>();
    let to = to.components().collect::<Vec<_>>();
    let mut i = 0;
    while i < from.len() && i < to.len() && from[i] == to[i] {
        i += 1;
    }
    let mut rel = PathBuf::new();
    for _ in i..from.len() {
        rel.push("..");
    }
    for part in &to[i..] {
        rel.push(part.as_os_str());
    }
    if rel.as_os_str().is_empty() {
        rel.push(".");
    }
    Some(rel)
}
