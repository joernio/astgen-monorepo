//! Utilities for working with Ungrammar grammars.

use ungrammar::{Grammar, Rule};

/// Return the names of all the rules, in the order they appear in the grammar.
pub(crate) fn collect_rule_names(grammar: &Grammar) -> Vec<String> {
    grammar
        .iter()
        .map(|node_key| grammar[node_key].name.clone())
        .collect()
}

/// Return all the token texts in the grammar.
/// There might be duplicate token texts, as the grammar can contain multiple rules with the same token.
pub(crate) fn collect_token_names(grammar: &Grammar) -> Vec<String> {
    grammar
        .iter()
        .flat_map(|node_key| tokens_inside_rule(grammar, &grammar[node_key].rule))
        .collect()
}

/// Return all the token texts in a rule.
/// May contain duplicates.
fn tokens_inside_rule(grammar: &Grammar, rule: &Rule) -> Vec<String> {
    match rule {
        Rule::Alt(rules) => rules
            .iter()
            .flat_map(|r| tokens_inside_rule(grammar, r))
            .collect(),
        Rule::Rep(rule) => tokens_inside_rule(grammar, rule),
        Rule::Labeled { rule, .. } => tokens_inside_rule(grammar, rule),
        Rule::Node(_) => vec![],
        Rule::Token(t) => vec![grammar[*t].name.clone()],
        Rule::Seq(rules) => rules
            .iter()
            .flat_map(|r| tokens_inside_rule(grammar, r))
            .collect(),
        Rule::Opt(rule) => tokens_inside_rule(grammar, rule),
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_rule_names, collect_token_names};
    use std::str::FromStr;
    use ungrammar::Grammar;

    #[test]
    fn test_collect_rule_names_1() {
        use std::str::FromStr;
        let g = Grammar::from_str("A = 'a'").unwrap();
        let rule_names = collect_rule_names(&g);
        assert_eq!(rule_names, vec!["A"]);
    }

    #[test]
    fn test_collect_rule_names_2() {
        let g = Grammar::from_str(
            r"
        A = 'a'
        B = 'b'
        ",
        )
        .unwrap();
        let rule_names = collect_rule_names(&g);
        assert_eq!(rule_names, vec!["A", "B"]);
    }

    #[test]
    fn test_collect_token_texts_1() {
        let g = Grammar::from_str("A = 'a'").unwrap();
        let token_texts = collect_token_names(&g);
        assert_eq!(token_texts, vec!["a"]);
    }

    #[test]
    fn test_collect_token_texts_2() {
        let g = Grammar::from_str(
            r"
        A = 'a'
        B = 'b' | 'b'
        ",
        )
        .unwrap();
        let token_texts = collect_token_names(&g);
        assert_eq!(token_texts, vec!["a", "b", "b"]);
    }
}
