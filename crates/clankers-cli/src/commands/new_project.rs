use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{bail, Context, Result};

pub fn execute(name: &str, template: &str) -> Result<()> {
    let template_dir = find_template(template)?;
    let dest = Path::new(name);

    if dest.exists() {
        bail!("directory '{}' already exists", name);
    }

    copy_dir_recursive(&template_dir, dest)?;
    replace_in_tree(dest, "{{PROJECT_NAME}}", name)?;

    let clankers_path = clankers_dependency_path(dest)?;
    replace_in_tree(dest, "{{CLANKERS_PATH}}", &clankers_path)?;

    println!("clankeRS project created: {name}\n\nNext steps:\n  cd {name}\n  clankers run");
    Ok(())
}

fn clankers_dependency_path(dest: &Path) -> Result<String> {
    if let Some(workspace) = find_workspace_root() {
        let clankers_crate = workspace.join("crates/clankers");
        if clankers_crate.exists() {
            let dest_abs = std::env::current_dir()?.join(dest);
            if let Some(rel) = relative_path(&dest_abs, &clankers_crate) {
                return Ok(rel.to_string_lossy().into_owned());
            }
        }
    }
    // Fallback for out-of-tree projects after `cargo install clankers-cli`
    Ok("../clankeRS/crates/clankers".to_string())
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
        if matches!(from[i], Component::Normal(_)) {
            rel.push("..");
        }
    }
    for part in &to[i..] {
        rel.push(part.as_os_str());
    }
    if rel.as_os_str().is_empty() {
        rel.push(".");
    }
    Some(rel)
}

fn find_workspace_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join("crates/clankers").exists() {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

fn find_template(template: &str) -> Result<std::path::PathBuf> {
    let normalized = template.replace('-', "_");
    let candidates = [
        format!("templates/{normalized}"),
        format!("../templates/{normalized}"),
        format!("../../templates/{normalized}"),
    ];

    for c in &candidates {
        let p = Path::new(c);
        if p.exists() {
            return Ok(p.canonicalize().unwrap_or_else(|_| p.to_path_buf()));
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let p = ancestor.join("templates").join(&normalized);
            if p.exists() {
                return Ok(p);
            }
        }
    }

    bail!(
        "template '{template}' not found; available: basic_node, perception_node, ml_inference_node, replay_test_node"
    )
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

fn replace_in_tree(dir: &Path, from: &str, to: &str) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let path = entry.path();
            let content = fs::read_to_string(path).context("read template file")?;
            if content.contains(from) {
                fs::write(path, content.replace(from, to))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_path_to_sibling_crate() {
        let from = Path::new("/workspace/clankeRS/hello_clanker");
        let to = Path::new("/workspace/clankeRS/crates/clankers");
        assert_eq!(
            relative_path(from, to).unwrap(),
            PathBuf::from("../crates/clankers")
        );
    }
}
