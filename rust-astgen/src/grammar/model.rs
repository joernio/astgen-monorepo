//! Sits between Ungrammar and the Scala bindings codegen, by
//! building a model the latter can more simply consume.

use crate::grammar::ungrammar::{collect_rule_names, collect_token_names};
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use ungrammar::Grammar;
use ungrammar::{Error, Rule};

pub struct Model {
    /// All node names in grammar order.
    node_names: Vec<NodeName>,

    /// Maps a node to its elements
    node_elements: HashMap<NodeName, Vec<Element>>,

    /// All the tokens found in the grammar.
    tokens: HashSet<TokenName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NodeName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TokenName(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ElementName {
    Node(NodeName),
    Token(TokenName),
}

impl NodeName {
    fn new(name: String) -> Self {
        Self(name)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for NodeName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl TokenName {
    fn new(name: String) -> Self {
        Self(name)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for TokenName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl ElementName {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            ElementName::Node(node) => node.as_str(),
            ElementName::Token(token) => token.as_str(),
        }
    }
}

/// A node in the RHS of a rule.
///
/// For instance, in:
///
/// ```text
/// ParenthesizedArgList =
///   '::'? '(' (TypeArg (',' TypeArg)* ','?)? ')'
/// ```
///
/// The node name is 'ParenthesizedArgList', and the elements are:
///
/// 1. "::" (Optional)
/// 2. "(" (One)
/// 3. TypeArg (Many)
/// 4. "," (Many)
/// 5. ")" (One)
///
pub(crate) struct Element {
    pub(crate) name: ElementName,
    pub(crate) cardinality: Cardinality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cardinality {
    One,
    Optional,
    Many,
}

impl Model {
    pub fn from_ungrammar(grammar: &Grammar) -> Result<Self, Error> {
        let node_names = collect_rule_names(grammar)
            .into_iter()
            .map(NodeName::new)
            .collect();
        let tokens = collect_token_names(grammar)
            .into_iter()
            .map(TokenName::new)
            .collect();
        let node_elements = grammar
            .iter()
            .map(|node_key| {
                let node_data = &grammar[node_key];
                let name = NodeName::new(node_data.name.to_owned());
                let all_elements = collect_elements(grammar, &node_data.rule, Cardinality::One);
                let merged_elements = merge_elements(all_elements, Cardinality::for_sequence);
                (name, merged_elements)
            })
            .collect();

        Ok(Self {
            node_names,
            node_elements,
            tokens,
        })
    }

    /// All node names
    pub(crate) fn nodes(&self) -> impl Iterator<Item = &str> {
        self.node_names.iter().map(NodeName::as_str)
    }

    /// All token texts
    pub(crate) fn tokens(&self) -> impl Iterator<Item = &str> {
        self.tokens.iter().map(TokenName::as_str)
    }

    /// The [Element]s of a node (rule).
    pub(crate) fn elements(&self, node: &str) -> Option<&[Element]> {
        self.node_elements.get(node).map(Vec::as_slice)
    }

    pub(crate) fn contains_node(&self, name: &str) -> bool {
        self.node_names.iter().any(|node| node.as_str() == name)
    }

    /// Only immediate child nodes (not tokens).
    fn immediate_child_nodes_of(&self, name: &str) -> HashSet<&str> {
        self.node_elements
            .get(name)
            .into_iter()
            .flatten()
            .filter_map(|element| match &element.name {
                ElementName::Node(node) => Some(node.as_str()),
                ElementName::Token(_) => None,
            })
            .collect()
    }

    /// Recursively collects child nodes (not tokens) starting from `name`.
    /// We can skip recurring into certain nodes with `should_recurse_on`.
    pub(crate) fn collect_descendant_nodes<F>(
        &self,
        name: &str,
        should_recurse_into: &F,
    ) -> HashSet<String>
    where
        F: Fn(&str) -> bool,
    {
        if !self.contains_node(name) {
            return HashSet::new();
        }

        if !should_recurse_into(name) {
            return HashSet::from([name.to_string()]);
        }

        self.immediate_child_nodes_of(name)
            .into_iter()
            .flat_map(|child| self.collect_descendant_nodes(child, should_recurse_into))
            .collect()
    }
}

/// Recursively collects the elements used by a rule.
/// May contain duplicates. See [merge_elements] for deduplication.
/// `cardinality` is the enclosing one, since a rule may contain other rules.
fn collect_elements(grammar: &Grammar, rule: &Rule, cardinality: Cardinality) -> Vec<Element> {
    match rule {
        Rule::Node(node) => vec![Element {
            name: ElementName::Node(NodeName::new(grammar[*node].name.to_owned())),
            cardinality,
        }],
        Rule::Token(token) => vec![Element {
            name: ElementName::Token(TokenName::new(grammar[*token].name.to_owned())),
            cardinality,
        }],
        Rule::Seq(rules) => rules
            .iter()
            .flat_map(|rule| collect_elements(grammar, rule, cardinality))
            .collect(),
        Rule::Alt(rules) => rules
            .iter()
            // Alts need to be merged before merging their elements with the enclosing context.
            // Otherwise, multiple same-named-elements-but-with-different-cardinalities could be
            // output, and in merge_elements we just look for the cardinality of the first match.
            // Could potentially be rewritten so that merging really only happens once at the end,
            // but this seems a good compromise for now.
            .map(|rule| {
                merge_elements(
                    collect_elements(grammar, rule, cardinality),
                    Cardinality::for_sequence,
                )
            })
            .reduce(merge_alternatives)
            .unwrap_or_default(),
        Rule::Opt(rule) => collect_elements(grammar, rule, cardinality.optional()),
        Rule::Rep(rule) => collect_elements(grammar, rule, Cardinality::Many),
        Rule::Labeled { rule, .. } => collect_elements(grammar, rule, cardinality),
    }
}

/// Traverse the elements merging those with the same name, combining their cardinalities.
fn merge_elements(
    elements: Vec<Element>,
    join: fn(Cardinality, Cardinality) -> Cardinality,
) -> Vec<Element> {
    elements
        .into_iter()
        .fold(Vec::new(), |mut merged, element| {
            let existing_with_same_name = merged.iter_mut().find(|e| e.name == element.name);

            match existing_with_same_name {
                None => merged.push(element),
                Some(existing) => {
                    existing.cardinality = join(existing.cardinality, element.cardinality)
                }
            }

            merged
        })
}

/// Returns the cardinality of the given element (by name) if it exists.
fn find_cardinality_of(elems: &[Element], name: &ElementName) -> Option<Cardinality> {
    elems
        .iter()
        .find(|e| e.name == *name)
        .map(|e| e.cardinality)
}

/// Merges two branches of an alternative, i.e. remove duplicate elements.
/// If an element appears on both sides, combine their cardinalities.
/// If an element appears on only one side, mark it optional.
fn merge_alternatives(left: Vec<Element>, right: Vec<Element>) -> Vec<Element> {
    let mut result = Vec::new();

    for left_elem in &left {
        let name = left_elem.name.to_owned();

        let cardinality = match find_cardinality_of(&right, &name) {
            Some(right_card) => left_elem.cardinality.for_alternative(right_card),
            None => left_elem.cardinality.optional(),
        };

        result.push(Element { name, cardinality })
    }

    for right_elem in &right {
        let name = right_elem.name.to_owned();

        if find_cardinality_of(&left, &name).is_none() {
            result.push(Element {
                name,
                cardinality: right_elem.cardinality.optional(),
            })
        }
    }

    result
}

impl Cardinality {
    fn for_sequence(self, other: Self) -> Self {
        use Cardinality::*;
        match (self, other) {
            (Optional, Optional) => Optional,
            _ => Many,
        }
    }

    fn for_alternative(self, other: Self) -> Self {
        use Cardinality::*;
        match (self, other) {
            (Many, _) => Many,
            (_, Many) => Many,
            (One, One) => One,
            _ => Optional,
        }
    }

    fn optional(self) -> Self {
        use Cardinality::*;
        match self {
            Many => Many,
            _ => Optional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn find_element<'a>(elements: &'a [Element], name: &str) -> &'a Element {
        elements.iter().find(|e| e.name.as_str() == name).unwrap()
    }

    #[test]
    fn test_1_node_2_tokens() {
        let grammar = Grammar::from_str("Name = '#ident' | 'self'").unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();

        let node_names = model.nodes().collect::<Vec<_>>();

        assert_eq!(node_names, vec!["Name"]);
        assert_eq!(model.tokens().count(), 2);
        assert!(model.tokens().any(|token| token == "#ident"));
        assert!(model.tokens().any(|token| token == "self"));

        let elements = model.elements("Name").unwrap();
        assert_eq!(elements.len(), 2);
        assert!(
            elements
                .iter()
                .any(|e| e.name.as_str() == "#ident" && e.cardinality == Cardinality::Optional)
        );
        assert!(
            elements
                .iter()
                .any(|e| e.name.as_str() == "self" && e.cardinality == Cardinality::Optional)
        );
    }

    #[test]
    fn test_collect_descendant_nodes_1() {
        let grammar = Grammar::from_str(
            r#"
            Stmt = Expr | Item | Let
            Expr = Literal
            Item = Fn | Struct
            Let = 'let'
            Literal = 'literal'
            Fn = 'fn'
            Struct = 'struct'
            "#,
        )
        .unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();

        assert_eq!(
            model
                .collect_descendant_nodes("Stmt", &|name| matches!(name, "Stmt" | "Expr" | "Item")),
            vec![
                "Fn".to_string(),
                "Let".to_string(),
                "Literal".to_string(),
                "Struct".to_string(),
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn test_rust_parenthesized_arg_list() {
        let grammar = Grammar::from_str(include_str!("../../rust.ungram")).unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();
        let elements = model.elements("ParenthesizedArgList").unwrap();

        // ParenthesizedArgList =
        //   '::'? '(' (TypeArg (',' TypeArg)* ','?)? ')'

        // Expected:
        // 1. '::' (Optional)
        // 2. '(' (One)
        // 3. TypeArg (Many)
        // 4. ',' (Many)
        // 5. ')' (One)

        assert_eq!(
            find_element(elements, "::").cardinality,
            Cardinality::Optional
        );
        assert_eq!(find_element(elements, "(").cardinality, Cardinality::One);
        assert_eq!(
            find_element(elements, "TypeArg").cardinality,
            Cardinality::Many
        );
        assert_eq!(find_element(elements, ",").cardinality, Cardinality::Many);
        assert_eq!(find_element(elements, ")").cardinality, Cardinality::One);
    }

    #[test]
    fn test_rust_ref_expr() {
        let grammar = Grammar::from_str(include_str!("../../rust.ungram")).unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();
        let elements = model.elements("RefExpr").unwrap();

        // RefExpr =
        //   Attr* '&' (('raw' 'const'?)| ('raw'? 'mut') ) Expr

        assert_eq!(
            find_element(elements, "Attr").cardinality,
            Cardinality::Many
        );
        assert_eq!(find_element(elements, "&").cardinality, Cardinality::One);
        assert_eq!(
            find_element(elements, "raw").cardinality,
            Cardinality::Optional
        );
        assert_eq!(
            find_element(elements, "const").cardinality,
            Cardinality::Optional
        );
        assert_eq!(
            find_element(elements, "mut").cardinality,
            Cardinality::Optional
        );
        assert_eq!(find_element(elements, "Expr").cardinality, Cardinality::One);
    }

    #[test]
    fn test_rust_bin_expr() {
        let grammar = Grammar::from_str(include_str!("../../rust.ungram")).unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();
        let elements = model.elements("BinExpr").unwrap();

        // BinExpr =
        //   Attr*
        //   lhs:Expr
        //   op:(
        //     '||' | '&&'
        //   | '==' | '!=' | '<=' | '>=' | '<' | '>'
        //   | '+' | '*' | '-' | '/' | '%' | '<<' | '>>' | '^' | '|' | '&'
        //   | '=' | '+=' | '/=' | '*=' | '%=' | '>>=' | '<<=' | '-=' | '|=' | '&=' | '^='
        //   )
        //   rhs:Expr

        assert_eq!(
            find_element(elements, "Attr").cardinality,
            Cardinality::Many
        );
        assert_eq!(
            find_element(elements, "Expr").cardinality,
            Cardinality::Many
        );
        assert_eq!(
            find_element(elements, "||").cardinality,
            Cardinality::Optional
        );
    }

    #[test]
    fn test_rust_tuple_type() {
        let grammar = Grammar::from_str(include_str!("../../rust.ungram")).unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();
        let elements = model.elements("TupleType").unwrap();

        // TupleType =
        //   '(' fields:(Type (',' Type)* ','?)? ')'

        assert_eq!(
            find_element(elements, "Type").cardinality,
            Cardinality::Many
        );
    }

    #[test]
    fn test_rust_range_pat() {
        let grammar = Grammar::from_str(include_str!("../../rust.ungram")).unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();
        let elements = model.elements("RangePat").unwrap();

        // RangePat =
        //   // 1..
        //   start:Pat op:('..' | '..=')
        //   // 1..2
        //   | start:Pat op:('..' | '..=') end:Pat
        //   // ..2
        //   | op:('..' | '..=') end:Pat

        assert_eq!(find_element(elements, "Pat").cardinality, Cardinality::Many);
        assert_eq!(
            find_element(elements, "..").cardinality,
            Cardinality::Optional
        );
        assert_eq!(
            find_element(elements, "..=").cardinality,
            Cardinality::Optional
        );
    }
}
