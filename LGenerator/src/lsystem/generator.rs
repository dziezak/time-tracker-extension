use lsystem::{LSystem, LRules};
use std::collections::HashMap;

pub struct CustomRules {
    pub rules: HashMap<char, Vec<char>>,
}

impl LRules<char> for CustomRules {
    fn map(&self, var: &char) -> Option<Vec<char>> {
        self.rules.get(var).cloned()
    }
}

pub fn build_tree_sequence(depth: usize) -> String {
    let mut rules_map = HashMap::new();
    rules_map.insert('F', "FF-[-F+J]+[+F&J]/[^F\\J]".chars().collect::<Vec<_>>());

    let axiom = vec!['F'];
    let mut current = axiom;

    for _ in 0..depth {
        let mut next = Vec::new();
        for &ch in &current {
            if let Some(replacement) = rules_map.get(&ch) {
                next.extend_from_slice(replacement);
            } else {
                next.push(ch);
            }
        }
        current = next;
    }

    current.into_iter().collect()
}

use crate::data::parser::DomainData;

pub fn build_tree_sequence_from_domains(domains: &[DomainData]) -> String {
    let mut sequence = String::from("FF");

    let branch_rotations = [
        "&[+FJJJ-FJJJ]",
        "^[-FJJJ+FJJJ]",
        "/&[+FJJJ]",
        "\\^[+FJJJ]",
        "+&[-FJJJ]",
        "-^[+FJJJ]"
    ];

    for (i, _domain) in domains.iter().enumerate() {
        let rot = branch_rotations[i % branch_rotations.len()];
        sequence.push_str(&format!("[D{}{}]", i, rot));
    }

    sequence
}