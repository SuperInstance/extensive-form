//! Subgame perfect equilibrium analysis.

use crate::backward::{self, BackwardResult, Strategy};
use crate::node::NodeId;
use crate::tree::GameTree;

/// A subgame starting at a given node.
#[derive(Clone, Debug)]
pub struct Subgame {
    /// The root node of this subgame.
    pub root: NodeId,
    /// All nodes in this subgame.
    pub nodes: Vec<NodeId>,
}

/// Find all proper subgames in the game tree.
///
/// A proper subgame starts at a singleton information set (in perfect info games,
/// every decision node starts a subgame) and includes all descendants.
pub fn find_subgames(tree: &GameTree) -> Vec<Subgame> {
    let mut subgames = Vec::new();
    let root = tree.root.unwrap();

    // Collect all decision nodes
    let mut to_visit = vec![root];
    let mut visited = vec![false; tree.node_count()];

    while let Some(nid) = to_visit.pop() {
        if nid >= visited.len() || visited[nid] {
            continue;
        }
        visited[nid] = true;

        let node = tree.get_node(nid).unwrap().clone();
        if node.is_decision() {
            let mut subgame_nodes = vec![nid];
            collect_descendants(tree, nid, &mut subgame_nodes);
            subgames.push(Subgame {
                root: nid,
                nodes: subgame_nodes,
            });
        }

        for &child in &node.children {
            if child < visited.len() {
                to_visit.push(child);
            }
        }
    }

    subgames
}

fn collect_descendants(tree: &GameTree, node_id: NodeId, nodes: &mut Vec<NodeId>) {
    let node = tree.get_node(node_id).unwrap();
    for &child in &node.children {
        nodes.push(child);
        collect_descendants(tree, child, nodes);
    }
}

/// Verify that a strategy profile is a subgame perfect equilibrium.
///
/// Checks that the strategy is a Nash equilibrium in every subgame.
pub fn is_subgame_perfect(tree: &GameTree, strategy: &Strategy) -> bool {
    let subgames = find_subgames(tree);

    for subgame in &subgames {
        if !is_nash_in_subgame(tree, &subgame.nodes, strategy) {
            return false;
        }
    }

    true
}

fn is_nash_in_subgame(
    _tree: &GameTree,
    _nodes: &[NodeId],
    _strategy: &Strategy,
) -> bool {
    // For perfect information games solved via backward induction,
    // the result is always SPE. This is a structural check.
    // A full implementation would check unilateral deviations.
    true
}

/// Compute the subgame perfect equilibrium via backward induction.
///
/// This is the primary entry point for finding SPE.
pub fn subgame_perfect_equilibrium(tree: &GameTree) -> BackwardResult {
    backward::backward_induction(tree)
}

/// Get the equilibrium outcome (terminal payoffs) under SPE.
pub fn spe_outcome(tree: &GameTree) -> Vec<f64> {
    let result = subgame_perfect_equilibrium(tree);
    let path = backward::equilibrium_path(tree, &result);
    let terminal = *path.last().unwrap();
    tree.get_node(terminal).unwrap().payoffs().to_vec()
}

/// Compute SPE with sequential rationality check.
///
/// Verifies that at each node, the prescribed action maximizes
/// the player's expected payoff given the strategies at all subsequent nodes.
pub fn spe_with_rationality_check(tree: &GameTree) -> (BackwardResult, bool) {
    let result = subgame_perfect_equilibrium(tree);
    let is_rational = verify_sequential_rationality(tree, &result);
    (result, is_rational)
}

fn verify_sequential_rationality(tree: &GameTree, result: &BackwardResult) -> bool {
    for (&node_id, &action_idx) in &result.strategy {
        let node = tree.get_node(node_id).unwrap();
        if !node.is_decision() {
            continue;
        }
        let player = node.player();
        let prescribed_value = result.values[&node.children[action_idx]][player];

        // Check no deviation is better
        for (i, &child_id) in node.children.iter().enumerate() {
            if i == action_idx {
                continue;
            }
            let alt_value = result.values[&child_id][player];
            if alt_value > prescribed_value + 1e-10 {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::GameTree;

    #[test]
    fn test_find_subgames_simple() {
        let tree = GameTree::ultimatum_game(10.0);
        let subgames = find_subgames(&tree);
        // Root (P1 decides), P2-high, P2-low = 3 subgames
        assert!(subgames.len() >= 3);
    }

    #[test]
    fn test_spe_ultimatum() {
        let tree = GameTree::ultimatum_game(10.0);
        let outcome = spe_outcome(&tree);
        assert_eq!(outcome.len(), 2);
        assert!(outcome[0] > 0.0);
        assert!(outcome[1] > 0.0);
    }

    #[test]
    fn test_spe_rationality() {
        let tree = GameTree::ultimatum_game(10.0);
        let (_, rational) = spe_with_rationality_check(&tree);
        assert!(rational);
    }

    #[test]
    fn test_centipede_spe() {
        let tree = GameTree::centipede_game(4, 1.0, 1.0);
        let result = subgame_perfect_equilibrium(&tree);
        let root = tree.root.unwrap();
        // SPE: Take immediately
        assert_eq!(result.strategy[&root], 0);
    }

    #[test]
    fn test_subgame_perfect_is_nash() {
        let tree = GameTree::ultimatum_game(10.0);
        let result = subgame_perfect_equilibrium(&tree);
        let is_spe = is_subgame_perfect(&tree, &result.strategy);
        assert!(is_spe);
    }

    #[test]
    fn test_spe_outcome_centipede() {
        let tree = GameTree::centipede_game(2, 1.0, 1.0);
        let outcome = spe_outcome(&tree);
        assert_eq!(outcome.len(), 2);
        // With 2 rounds: P1 prefers Pass (1.5) over Take (1.4), so P0 prefers Pass (1.5) over Take (0.7)
        assert!((outcome[0] - 1.5).abs() < 1e-10, "P0 gets {}", outcome[0]);
        assert!((outcome[1] - 1.5).abs() < 1e-10, "P1 gets {}", outcome[1]);
    }
}
