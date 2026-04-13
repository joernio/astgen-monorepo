use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RustAstGenJsonNode {
    pub(crate) node_kind: String,
    pub(crate) start_offset: u32,
    pub(crate) end_offset: u32,
    pub(crate) start_line: u32,
    pub(crate) start_column: u32,
    pub(crate) children: Vec<RustAstGenJsonNode>,
}
