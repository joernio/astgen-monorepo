use ra_ap_hir::{ModuleDef, PathResolution, Semantics};
use std::ops::Range;

use crate::names::type_formatter;
use ra_ap_ide::RootDatabase;
use ra_ap_syntax::{AstNode, AstToken, SyntaxNode, ast};

pub(crate) struct ImplicitFormatArg {
    pub(crate) name: String,
    pub(crate) type_full_name: Option<String>,
}

pub(crate) fn implicit_format_args_for_node(
    node: &SyntaxNode,
    semantics: &Semantics<RootDatabase>,
) -> Option<Vec<ImplicitFormatArg>> {
    let format_args_expr = ast::FormatArgsExpr::cast(node.clone())?;
    let ast::Expr::Literal(template) = format_args_expr.template()? else {
        return None;
    };
    let ast::LiteralKind::String(string) = template.kind() else {
        return None;
    };

    let explicit_arg_names: Vec<String> = format_args_expr
        .args()
        .filter_map(|arg| arg.arg_name())
        .map(|arg_name| arg_name.name().text().to_owned())
        .collect();

    let module = semantics.scope(node)?.module();
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
            type_formatter::format_type(&typ, module, semantics.db)
        });

        captures.push(ImplicitFormatArg {
            name: name.to_owned(),
            type_full_name,
        });
    }

    Some(captures)
}
