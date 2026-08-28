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

    let rules = CustomRules { rules: rules_map };
    let axiom = vec!['F'];

    let mut system = LSystem::new(rules, axiom);

    let mut result = vec!['F'];
    for _ in 0..depth {
        if let Some(next_state) = system.next() {
            result = next_state;
        }
    }

    result.into_iter().collect()
}