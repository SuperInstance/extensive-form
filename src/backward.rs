//! Backward induction solver for extensive-form games.

use crate::node::{NodeId, NodeType};
use crate::tree::GameTree;
use std::collections::HashMap;

/// Result of backward induction: optimal action at each decision node.
pub type Strategy = HashMap<NodeId, usize>;

/// Result of backward induction: optimal action and expected payoff.
#[derive(Clone, Debug)]
pub struct BackwardResult {
    /// Optimal action index at each decision node.
    pub strategy: Strategy,
    /// Expected payoffs at each node after solving.
    pub values: HashMap<NodeId, Vec<f64>>,
}

/// Solve an extensive-form game via backward induction.
///
/// Returns the subgame-perfect equilibrium strategy and expected payoffs.
pub fn backward_induction(tree: &GameTree) -> BackwardResult {
    let mut values: HashMap<NodeId, Vec<f64>> = HashMap::new();
    let mut strategy: Strategy = HashMap::new();

    // Find root
    let root = tree.root.expect("Game tree must have a root");

    // Recursively solve from root
    solve_node(tree, root, &mut values, &mut strategy);

    BackwardResult { strategy, values }
}

fn solve_node(
    tree: &GameTree,
    node_id: NodeId,
    values: &mut HashMap<NodeId, Vec<f64>>,
    strategy: &mut Strategy,
) -> Vec<f64> {
    // Check if already solved
    if let Some(v) = values.get(&node_id) {
        return v.clone();
    }

    let node = tree.get_node(node_id).expect("Node must exist").clone();

    let result = match &node.node_type {
        NodeType::Terminal { payoffs } => payoffs.clone(),
        NodeType::Decision { player } => {
            let p = *player;
            let mut best_action = 0;
            let mut best_value = Vec::new();
            let mut best_player_val = f64::NEG_INFINITY;

            for (i, &child_id) in node.children.iter().enumerate() {
                let child_val = solve_node(tree, child_id, values, strategy);
                if child_val[p] > best_player_val {
                    best_player_val = child_val[p];
                    best_value = child_val;
                    best_action = i;
                }
            }

            strategy.insert(node_id, best_action);
            best_value
        }
        NodeType::Chance { probabilities } => {
            let n_players = tree.num_players;
            let mut expected = vec![0.0; n_players];

            for (i, &prob) in probabilities.iter().enumerate() {
                if i < node.children.len() {
                    let child_val = solve_node(tree, node.children[i], values, strategy);
                    for (p, ev) in expected.iter_mut().enumerate() {
                        if p < child_val.len() {
                            *ev += prob * child_val[p];
                        }
                    }
                }
            }

            expected
        }
    };

    values.insert(node_id, result.clone());
    result
}

/// Get the equilibrium path from root to terminal under backward induction.
pub fn equilibrium_path(tree: &GameTree, result: &BackwardResult) -> Vec<NodeId> {
    let mut path = Vec::new();
    let mut current = tree.root;

    while let Some(node_id) = current {
        path.push(node_id);
        let node = tree.get_node(node_id).unwrap();
        if node.is_terminal() {
            break;
        }
        match result.strategy.get(&node_id) {
            Some(&action_idx) => {
                current = node.children.get(action_idx).copied();
            }
            None => {
                // Chance node or unresolved: take first child
                current = node.children.first().copied();
            }
        }
    }

    path
}

/// Get the action labels along the equilibrium path.
pub fn equilibrium_actions(tree: &GameTree, result: &BackwardResult) -> Vec<String> {
    let path = equilibrium_path(tree, result);
    let mut actions = Vec::new();
    for &node_id in &path[1..] {
        if let Some(node) = tree.get_node(node_id) {
            if let Some(ref action) = node.incoming_action {
                actions.push(action.clone());
            }
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::GameTree;

    #[test]
    fn test_simple_backward() {
        let mut tree = GameTree::new("Simple");
        let root = tree.add_decision(0, "Choose");
        let left = tree.add_terminal(vec![1.0, 3.0], "L");
        let right = tree.add_terminal(vec![2.0, 1.0], "R");
        tree.add_action(root, "L", left);
        tree.add_action(root, "R", right);

        let result = backward_induction(&tree);
        // P0 prefers R (payoff 2 > 1)
        assert_eq!(result.strategy[&root], 1);
        let root_val = &result.values[&root];
        assert!((root_val[0] - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_ultimatum_backward() {
        let tree = GameTree::ultimatum_game(10.0);
        let result = backward_induction(&tree);
        // P2 should accept both offers (positive > 0)
        // P1 should offer low (keeps more)
        let root = tree.root.unwrap();
        let root_val = &result.values[&root];
        // P1 gets at least some positive payoff
        assert!(root_val[0] > 0.0);
    }

    #[test]
    fn test_centipede_backward() {
        let tree = GameTree::centipede_game(3, 1.0, 1.0);
        let result = backward_induction(&tree);
        let root = tree.root.unwrap();
        // In centipede, backward induction says Take immediately
        assert_eq!(result.strategy[&root], 0); // Take
    }

    #[test]
    fn test_equilibrium_path() {
        let mut tree = GameTree::new("Path");
        let root = tree.add_decision(0, "Root");
        let left = tree.add_terminal(vec![5.0, 0.0], "L");
        let right = tree.add_terminal(vec![0.0, 5.0], "R");
        tree.add_action(root, "L", left);
        tree.add_action(root, "R", right);

        let result = backward_induction(&tree);
        let path = equilibrium_path(&tree, &result);
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], root);
        assert_eq!(path[1], left); // P0 chooses L for payoff 5
    }

    #[test]
    fn test_multi_level_backward() {
        let mut tree = GameTree::new("Two Level");
        let root = tree.add_decision(0, "P0");
        let left = tree.add_decision(1, "P1-Left");
        let right = tree.add_decision(1, "P1-Right");
        let ll = tree.add_terminal(vec![3.0, 1.0], "LL");
        let lr = tree.add_terminal(vec![0.0, 2.0], "LR");
        let rl = tree.add_terminal(vec![2.0, 2.0], "RL");
        let rr = tree.add_terminal(vec![1.0, 3.0], "RR");

        tree.add_action(root, "L", left);
        tree.add_action(root, "R", right);
        tree.add_action(left, "L", ll);
        tree.add_action(left, "R", lr);
        tree.add_action(right, "L", rl);
        tree.add_action(right, "R", rr);

        let result = backward_induction(&tree);
        // P1 at left: prefers R (payoff 2 > 1) → (0, 2)
        // P1 at right: prefers R (payoff 3 > 2) → (1, 3)
        // P0 at root: prefers right path (payoff 1 > 0) → (1, 3)
        assert_eq!(result.strategy[&root], 1); // R
    }
}
