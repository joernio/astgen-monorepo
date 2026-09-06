//! The generated scala bindings and the JSON ast creation must agree on
//! `rust_ast_gen::json_ast::RustAstGenJsonNode` `node_kind` names.
//! To achieve so, they both get them from here.

use heck::ToShoutySnakeCase;
use ra_ap_syntax::{Edition, SyntaxKind};

pub fn syntax_kind_to_json_name(kind: SyntaxKind) -> String {
    format!("{kind:?}")
}

pub fn token_name_to_syntax_kind(token: &str) -> Option<SyntaxKind> {
    if let Some(kind) = all_kinds()
        .filter(|kind| kind.is_punct() || kind.is_keyword(Edition::LATEST))
        .find(|kind| kind.text() == token)
    {
        return Some(kind);
    }

    // Some tokens are "external", i.e. never defined in the ungrammar, but
    // referenced still, e.g. `#ident`, `@int_number`, etc.
    // Currently, removing their prefix is enough to resolve them.
    let external = token.strip_prefix(['#', '@'])?;
    kind_by_json_name(&external.to_shouty_snake_case())
}

pub fn node_name_to_syntax_kind(node: &str) -> Option<SyntaxKind> {
    kind_by_json_name(&node.to_shouty_snake_case())
}

fn all_kinds() -> impl Iterator<Item = SyntaxKind> {
    (0..SyntaxKind::__LAST as u16).map(SyntaxKind::from)
}

fn kind_by_json_name(name: &str) -> Option<SyntaxKind> {
    all_kinds().find(|kind| syntax_kind_to_json_name(*kind) == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ra_ap_syntax::T;

    #[test]
    fn punct_resolves() {
        assert_eq!(token_name_to_syntax_kind(";"), Some(T![;]));
        assert_eq!(token_name_to_syntax_kind("..="), Some(T![..=]));
    }

    #[test]
    fn keyword_resolves() {
        assert_eq!(token_name_to_syntax_kind("fn"), Some(T![fn]));
    }

    #[test]
    fn self_type_resolves() {
        assert_eq!(token_name_to_syntax_kind("Self"), Some(T![Self]));
        assert_eq!(token_name_to_syntax_kind("self"), Some(T![self]));
    }

    #[test]
    fn external_token_resolves() {
        assert_eq!(token_name_to_syntax_kind("#ident"), Some(SyntaxKind::IDENT));
        assert_eq!(
            token_name_to_syntax_kind("@int_number"),
            Some(SyntaxKind::INT_NUMBER)
        );
    }

    #[test]
    fn unknown_token_or_name_does_not_resolve() {
        assert_eq!(token_name_to_syntax_kind("foobar"), None);
        assert_eq!(node_name_to_syntax_kind("foobar"), None);
        assert_eq!(token_name_to_syntax_kind("#foobar"), None);
    }

    #[test]
    fn kind_inverses() {
        for kind in all_kinds() {
            assert_eq!(
                kind_by_json_name(&syntax_kind_to_json_name(kind)),
                Some(kind)
            );
        }
    }
}
