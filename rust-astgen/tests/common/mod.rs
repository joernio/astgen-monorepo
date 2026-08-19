//! Test fixture that runs `rust_ast_gen` (via cargo) against a temporary directory and
//! parses back the generated JSON. Currently only one crate is supported.
#![allow(dead_code)]

use serde_json::Value;
use std::fs;
use std::process::Command;
use temp_dir::TempDir;

pub type TestResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn no_sysroot_ast_json(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    target_file: &str,
) -> TestResult<Value> {
    run(crate_name, file_code_pairs, false, false, target_file)
}

pub fn no_sysroot_resolve_cfg_ast_json(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    target_file: &str,
) -> TestResult<Value> {
    run(crate_name, file_code_pairs, false, true, target_file)
}

pub fn no_sysroot_ast_json_generated(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    target_file: &str,
) -> bool {
    run(crate_name, file_code_pairs, false, false, target_file).is_ok()
}

pub fn sysroot_ast_json(crate_name: &str, source: &str) -> TestResult<Value> {
    run(
        crate_name,
        &[("src/main.rs", source)],
        true,
        false,
        "src/main.rs",
    )
}

pub fn sysroot_function_fullnames_json(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
) -> TestResult<Value> {
    function_fullnames_run(
        crate_name,
        file_code_pairs,
        true,
        &FunctionFullnamesRunOptions::default(),
    )
}

pub struct FunctionFullnamesRunOptions {
    pub target: Option<String>,
    pub features: Vec<String>,
    pub no_default_features: bool,
}

impl Default for FunctionFullnamesRunOptions {
    fn default() -> Self {
        Self {
            target: None,
            features: Vec::new(),
            no_default_features: false,
        }
    }
}

pub fn function_fullnames_by_crate(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    options: &FunctionFullnamesRunOptions,
) -> TestResult<Value> {
    function_fullnames_run(crate_name, file_code_pairs, true, options)
}

fn function_fullnames_run(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    with_sysroot: bool,
    options: &FunctionFullnamesRunOptions,
) -> TestResult<Value> {
    let root = TempDir::with_prefix("rust_ast_gen_function_fullnames_test_")?;

    fs::write(
        root.child("Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    )?;

    for (relative_path, content) in file_code_pairs {
        let path = root.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }

    let mut command = Command::new(env!("CARGO_BIN_EXE_rust_ast_function_fullnames"));
    command.arg("-i").arg(root.path());
    if !with_sysroot {
        command.arg("--no-sysroot");
    }
    if let Some(target) = &options.target {
        command.arg("--target").arg(target);
    }
    if !options.features.is_empty() {
        command.arg("--features").arg(options.features.join(","));
    }
    if options.no_default_features {
        command.arg("--no-default-features");
    }
    let output = command.output()?;

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(
        output.status.success(),
        "rust_ast_function_fullnames failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let parsed: Value = serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;
    Ok(parsed)
}

fn run(
    crate_name: &str,
    file_code_pairs: &[(&str, &str)],
    with_sysroot: bool,
    resolve_cfg: bool,
    target_file: &str,
) -> TestResult<Value> {
    // NB: automatically deleted when dropped
    let root = TempDir::with_prefix("rust_ast_gen_integration_test_")?;
    let output_dir = root.child("out");

    fs::write(
        root.child("Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2021"
"#
        ),
    )?;

    for (relative_path, content) in file_code_pairs {
        let path = root.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
    }

    let mut command = Command::new(env!("CARGO_BIN_EXE_rust_ast_gen"));
    command
        .arg("-i")
        .arg(root.path())
        .arg("-o")
        .arg(&output_dir);
    if !with_sysroot {
        command.arg("--no-sysroot");
    }
    if resolve_cfg {
        command.arg("--resolve-cfg");
    }
    let output = command.output()?;

    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    assert!(
        output.status.success(),
        "rust_ast_gen failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let target_json = output_dir.join(format!("{target_file}.json"));
    Ok(serde_json::from_str(&fs::read_to_string(&target_json)?)?)
}

pub fn nodes_by_kind<'a>(json: &'a Value, kind: &str) -> Vec<&'a Value> {
    let mut result = Vec::new();
    if let Some(children) = json.get("children").and_then(Value::as_array) {
        for child in children {
            collect_nodes_by_kind(child, kind, &mut result);
        }
    }
    result
}

fn collect_nodes_by_kind<'a>(node: &'a Value, kind: &str, result: &mut Vec<&'a Value>) {
    if node.get("nodeKind").and_then(Value::as_str) == Some(kind) {
        result.push(node);
    }

    if let Some(children) = node.get("children").and_then(Value::as_array) {
        for child in children {
            collect_nodes_by_kind(child, kind, result);
        }
    }

    // NB: macro-expanded nodes live under macroExpansion, not children.
    if let Some(expansion) = node.get("macroExpansion") {
        collect_nodes_by_kind(expansion, kind, result);
    }
}

fn node_text<'a>(json: &'a Value, node: &Value) -> Option<&'a str> {
    let content = json.get("content")?.as_str()?;
    let range = node.get("range")?;
    let start = range.get("startOffset")?.as_u64()? as usize;
    let end = range.get("endOffset")?.as_u64()? as usize;
    content.get(start..end)
}

#[derive(Clone, Copy)]
pub struct NodeSelector<'a> {
    json: &'a Value,
    kind: &'static str,
    text: &'static str,
    line: Option<&'static str>,
}

impl<'a> NodeSelector<'a> {
    pub fn on_line(mut self, line: &'static str) -> Self {
        self.line = Some(line);
        self
    }

    pub fn exists(self) -> bool {
        !self.nodes().is_empty()
    }

    pub fn type_full_name(self) -> String {
        self.field("typeFullName")
    }

    pub fn method_full_name(self) -> String {
        self.field("methodFullName")
    }

    pub fn method_full_name_opt(self) -> Option<String> {
        self.one_node()
            .get("methodFullName")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }

    fn field(self, field: &str) -> String {
        self.one_node()
            .get(field)
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("expected {field} on {}", self.description()))
            .to_owned()
    }

    fn one_node(self) -> &'a Value {
        let nodes = self.nodes();
        assert_eq!(
            nodes.len(),
            1,
            "expected exactly one {}, found {}: {:?}",
            self.description(),
            nodes.len(),
            nodes
                .iter()
                .map(|node| node_text(self.json, node))
                .collect::<Vec<_>>()
        );
        nodes[0]
    }

    fn nodes(self) -> Vec<&'a Value> {
        nodes_by_kind(self.json, self.kind)
            .into_iter()
            .filter(|node| self.matches_line(node) && node_text(self.json, node) == Some(self.text))
            .collect()
    }

    fn matches_line(self, node: &Value) -> bool {
        self.line.is_none_or(|expected| {
            node_start_line(self.json, node).is_some_and(|line| line.trim() == expected.trim())
        })
    }

    fn description(self) -> String {
        let mut parts = vec![format!("{} `{}`", self.kind, self.text)];
        if let Some(line) = self.line {
            parts.push(format!("on line `{}`", line.trim()));
        }
        parts.join(" ")
    }

    pub fn adjustments(self) -> Vec<Value> {
        self.one_node()
            .get("adjustments")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    }

    pub fn has_self_receiver(self) -> Option<bool> {
        self.one_node()
            .get("hasSelfReceiver")
            .and_then(Value::as_bool)
    }

    pub fn implemented_traits(self) -> Vec<String> {
        self.string_list("implementedTraits")
    }

    pub fn supertraits(self) -> Vec<String> {
        self.string_list("supertraits")
    }

    fn string_list(self, field: &str) -> Vec<String> {
        self.one_node()
            .get(field)
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn node<'a>(json: &'a Value, kind: &'static str, text: &'static str) -> NodeSelector<'a> {
    NodeSelector {
        json,
        kind,
        text,
        line: None,
    }
}

pub fn call_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "CALL_EXPR", text)
}

pub fn method_call_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "METHOD_CALL_EXPR", text)
}

pub fn name_ref<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "NAME_REF", text)
}

pub fn ident_pat<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "IDENT_PAT", text)
}

pub fn struct_decl<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "STRUCT", text)
}

pub fn enum_decl<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "ENUM", text)
}

pub fn trait_decl<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "TRAIT", text)
}

pub fn fn_decl<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "FN", text)
}

pub fn self_param<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "SELF_PARAM", text)
}

pub fn literal<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "LITERAL", text)
}

pub fn bin_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "BIN_EXPR", text)
}

fn node_start_line<'a>(json: &'a Value, node: &Value) -> Option<&'a str> {
    let content = json.get("content")?.as_str()?;
    let start_line = node
        .get("range")?
        .get("startLine")?
        .as_u64()
        .and_then(|line| usize::try_from(line).ok())?;
    content.lines().nth(start_line)
}

pub fn path_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "PATH_EXPR", text)
}

pub fn ref_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "REF_EXPR", text)
}

pub fn return_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "RETURN_EXPR", text)
}

pub fn closure_expr<'a>(json: &'a Value, text: &'static str) -> NodeSelector<'a> {
    node(json, "CLOSURE_EXPR", text)
}
