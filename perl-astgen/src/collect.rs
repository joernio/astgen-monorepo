use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn collect_perl_files(
    input: &Path,
    exclude_files: &[PathBuf],
    exclude_regex: Option<&Regex>,
) -> Vec<PathBuf> {
    if input.is_file() {
        let ext = input.extension().and_then(|e| e.to_str());
        if !matches!(ext, Some("pl") | Some("pm")) {
            return vec![];
        }
        let Ok(canonical) = input.canonicalize() else {
            return vec![];
        };
        if exclude_files.iter().any(|ef| ef == &canonical) {
            return vec![];
        }
        if let Some(re) = exclude_regex {
            if re.is_match(canonical.to_str().unwrap_or("")) {
                return vec![];
            }
        }
        return vec![canonical];
    }

    WalkDir::new(input)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let ext = path.extension().and_then(|e| e.to_str());
            if !matches!(ext, Some("pl") | Some("pm")) {
                return None;
            }
            let canonical = path.canonicalize().ok()?;
            if exclude_files.iter().any(|ef| ef == &canonical) {
                return None;
            }
            if let Some(re) = exclude_regex {
                if re.is_match(canonical.to_str().unwrap_or("")) {
                    return None;
                }
            }
            Some(canonical)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_tree(base: &Path, files: &[&str]) {
        for f in files {
            let p = base.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, b"1;").unwrap();
        }
    }

    #[test]
    fn collects_pl_and_pm_files_recursively() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["a.pl", "sub/b.pm", "sub/c.txt", "d.py"]);
        let files = collect_perl_files(dir.path(), &[], None);
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names.contains(&"a.pl"));
        assert!(names.contains(&"b.pm"));
        assert!(!names.contains(&"c.txt"));
        assert!(!names.contains(&"d.py"));
    }

    #[test]
    fn exclude_file_removes_exact_match() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["a.pl", "b.pl"]);
        let exclude = vec![dir.path().join("a.pl").canonicalize().unwrap()];
        let files = collect_perl_files(dir.path(), &exclude, None);
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(!names.contains(&"a.pl"));
        assert!(names.contains(&"b.pl"));
    }

    #[test]
    fn exclude_regex_removes_matching_paths() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["foo_test.pl", "bar.pl"]);
        let re = Regex::new("_test\\.pl$").unwrap();
        let files = collect_perl_files(dir.path(), &[], Some(&re));
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(!names.contains(&"foo_test.pl"));
        assert!(names.contains(&"bar.pl"));
    }

    #[test]
    fn single_file_input_returns_only_that_file() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["only.pl", "sibling.pl", "other.pm"]);
        let path = dir.path().join("only.pl");
        let files = collect_perl_files(&path, &[], None);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap(), "only.pl");
    }

    #[test]
    fn single_file_input_wrong_extension_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["script.rb"]);
        let path = dir.path().join("script.rb");
        let files = collect_perl_files(&path, &[], None);
        assert!(files.is_empty());
    }

    #[test]
    fn single_file_input_excluded_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        make_tree(dir.path(), &["a.pl"]);
        let path = dir.path().join("a.pl");
        let exclude = vec![path.canonicalize().unwrap()];
        let files = collect_perl_files(&path, &exclude, None);
        assert!(files.is_empty());
    }
}
