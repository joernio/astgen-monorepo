//! Sits between Ungrammar and the Scala bindings codegen, by
//! building a model the latter can more simply consume.

use crate::grammar::ungrammar::{collect_rule_names, collect_token_names};
use std::collections::{HashMap, HashSet};
use ungrammar::Grammar;
use ungrammar::{Error, Rule};

pub struct Model {
    /// All node names in grammar order.
    pub(crate) node_names: Vec<String>,

    /// Maps a node to its elements
    pub(crate) node_elements: HashMap<String, Vec<Element>>,

    /// All the tokens found in the grammar.
    pub(crate) tokens: HashSet<String>,
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
    pub(crate) node_or_token_name: String,
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
        let node_names = collect_rule_names(grammar);
        let tokens = collect_token_names(grammar).into_iter().collect();
        let node_elements = grammar
            .iter()
            .map(|node_key| {
                let node_data = &grammar[node_key];
                let name = node_data.name.clone();
                let all_elements = elements(grammar, &node_data.rule, Cardinality::One);
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

    fn contains_node(&self, name: &str) -> bool {
        self.node_names.iter().any(|node| node == name)
    }

    /// Only immediate child nodes (not tokens).
    fn immediate_child_nodes_of(&self, name: &str) -> HashSet<&str> {
        self.node_elements
            .get(name)
            .into_iter()
            .flatten()
            .map(|element| element.node_or_token_name.as_str())
            .filter(|child| self.contains_node(child))
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

        let descendants = self
            .immediate_child_nodes_of(name)
            .into_iter()
            .flat_map(|child| self.collect_descendant_nodes(child, should_recurse_into))
            .collect();

        descendants
    }
}

/// Recursively collects the elements used by a rule.
/// May contain duplicates. See [merge_elements] for deduplication.
/// `cardinality` is the enclosing one, since a rule may contain other rules.
fn elements(grammar: &Grammar, rule: &Rule, cardinality: Cardinality) -> Vec<Element> {
    match rule {
        Rule::Node(node) => vec![Element {
            node_or_token_name: grammar[*node].name.clone(),
            cardinality,
        }],
        Rule::Token(token) => vec![Element {
            node_or_token_name: grammar[*token].name.clone(),
            cardinality,
        }],
        Rule::Seq(rules) => rules
            .iter()
            .flat_map(|rule| elements(grammar, rule, cardinality))
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
                    elements(grammar, rule, cardinality),
                    Cardinality::for_sequence,
                )
            })
            .reduce(merge_alternatives)
            .unwrap_or_default(),
        Rule::Opt(rule) => elements(grammar, rule, cardinality.optional()),
        Rule::Rep(rule) => elements(grammar, rule, Cardinality::Many),
        Rule::Labeled { rule, .. } => elements(grammar, rule, cardinality),
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
            let existing_with_same_name = merged
                .iter_mut()
                .find(|e| e.node_or_token_name == element.node_or_token_name);

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
fn find_cardinality_of(elems: &[Element], name: &str) -> Option<Cardinality> {
    elems
        .iter()
        .find(|e| e.node_or_token_name == name)
        .map(|e| e.cardinality)
}

/// Merges two branches of an alternative, i.e. remove duplicate elements.
/// If an element appears on both sides, combine their cardinalities.
/// If an element appears on only one side, mark it optional.
fn merge_alternatives(left: Vec<Element>, right: Vec<Element>) -> Vec<Element> {
    let mut result = Vec::new();

    for left_elem in &left {
        let node_or_token_name = left_elem.node_or_token_name.clone();

        let cardinality = match find_cardinality_of(&right, &node_or_token_name) {
            Some(right_card) => left_elem.cardinality.for_alternative(right_card),
            None => left_elem.cardinality.optional(),
        };

        result.push(Element {
            node_or_token_name,
            cardinality,
        })
    }

    for right_elem in &right {
        let node_or_token_name = right_elem.node_or_token_name.clone();

        if find_cardinality_of(&left, &node_or_token_name).is_none() {
            result.push(Element {
                node_or_token_name,
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
        elements
            .iter()
            .find(|e| e.node_or_token_name == name)
            .unwrap()
    }
    #[test]
    fn test_1_node_2_tokens() {
        let grammar = Grammar::from_str("Name = '#ident' | 'self'").unwrap();
        let model = Model::from_ungrammar(&grammar).unwrap();

        assert_eq!(model.node_names.len(), 1);
        assert_eq!(model.tokens.len(), 2);

        assert_eq!(model.node_names[0], "Name");
        assert!(model.tokens.contains("#ident"));
        assert!(model.tokens.contains("self"));

        let elements = &model.node_elements["Name"];
        assert_eq!(elements.len(), 2);
        assert!(elements
            .iter()
            .any(|e| e.node_or_token_name == "#ident" && e.cardinality == Cardinality::Optional));
        assert!(
            elements
                .iter()
                .any(|e| e.node_or_token_name == "self" && e.cardinality == Cardinality::Optional)
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
        let elements = &model.node_elements["ParenthesizedArgList"];

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
        let elements = &model.node_elements["RefExpr"];

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
        let elements = &model.node_elements["BinExpr"];

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
        let elements = &model.node_elements["TupleType"];

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
        let elements = &model.node_elements["RangePat"];

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
