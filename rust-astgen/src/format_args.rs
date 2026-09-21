use ra_ap_hir::{ModuleDef, PathResolution, Semantics};
use std::ops::Range;

use joern_fmt::type_formatter;
use ra_ap_ide_db::RootDatabase;
use ra_ap_syntax::{AstNode, AstToken, ast};

pub(crate) struct ImplicitFormatArg {
    pub(crate) name: String,
    pub(crate) type_full_name: Option<String>,
    pub(crate) format_spec: Option<String>,
}

pub(crate) fn implicit_format_args(
    format_args_expr: &ast::FormatArgsExpr,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<ImplicitFormatArg>> {
    let string = rust_analyzer_ext::format_template(format_args_expr)?;

    let explicit_arg_names: Vec<String> = format_args_expr
        .args()
        .filter_map(|arg| arg.arg_name())
        .map(|arg_name| arg_name.name().text().to_owned())
        .collect();

    let module = semantics.scope(format_args_expr.syntax())?.module();
    let literal_start = string.syntax().text_range().start();
    let literal_text = string.syntax().text();

    let mut captures: Vec<ImplicitFormatArg> = Vec::new();
    for (range, resolution) in semantics.as_format_args_parts(&string)? {
        let name = literal_text.get(Range::<usize>::from(range - literal_start))?;
        if explicit_arg_names.iter().any(|already| already == name)
            || captures.iter().any(|capture| capture.name == name)
        {
            continue;
        }

        let type_full_name = resolution.and_then(|resolution| {
            let typ = match resolution.left()? {
                PathResolution::Local(local) => local.ty(semantics.db),
                PathResolution::Def(ModuleDef::Const(konst)) => konst.ty(semantics.db),
                PathResolution::Def(ModuleDef::Static(statik)) => statik.ty(semantics.db),
                _ => return None,
            };
            type_formatter::format_type(&typ, module, semantics)
        });

        let format_spec = first_format_spec(&string, None, Some(name));

        captures.push(ImplicitFormatArg {
            name: name.to_owned(),
            type_full_name,
            format_spec,
        });
    }

    Some(captures)
}

pub(crate) fn format_spec(arg: &ast::FormatArgsArg) -> Option<String> {
    let format_args_expr = ast::FormatArgsExpr::cast(arg.syntax().parent()?)?;
    let string = rust_analyzer_ext::format_template(&format_args_expr)?;
    let index = format_args_expr
        .args()
        .position(|format_args_arg| &format_args_arg == arg)?;
    let name = arg
        .arg_name()
        .map(|arg_name| arg_name.name().text().to_owned());
    first_format_spec(&string, Some(index), name.as_deref())
}

fn first_format_spec(
    string: &ast::String,
    index: Option<usize>,
    name: Option<&str>,
) -> Option<String> {
    let placeholder = rust_analyzer_ext::format_placeholders(string)?
        .into_iter()
        .find(|placeholder| match &placeholder.argument {
            rust_analyzer_ext::FormatArgument::Named(arg_name) => Some(arg_name.as_str()) == name,
            rust_analyzer_ext::FormatArgument::Index(arg_index) => Some(*arg_index) == index,
        })?;
    let (_, spec) = string.text()[placeholder.range]
        .strip_suffix('}')?
        .split_once(':')?;
    Some(spec.to_owned())
}
