//! Information sets for modeling imperfect information.

use crate::node::NodeId;
use crate::tree::GameTree;
use std::collections::HashMap;

/// An information set groups nodes that a player cannot distinguish between.
#[derive(Clone, Debug)]
pub struct InformationSet {
    /// Unique identifier for this information set.
    pub id: usize,
    /// The player who owns this information set.
    pub player: usize,
    /// The nodes in this information set.
    pub nodes: Vec<NodeId>,
    /// Available actions (shared across all nodes in the set).
    pub actions: Vec<String>,
}

/// Collection of all information sets in a game.
#[derive(Clone, Debug, Default)]
pub struct InformationStructure {
    /// Information sets indexed by their ID.
    pub sets: Vec<InformationSet>,
    /// Map from node ID to information set ID.
    pub node_to_set: HashMap<NodeId, usize>,
}

impl InformationStructure {
    /// Create a new empty information structure.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an information set.
    pub fn add_set(
        &mut self,
        player: usize,
        nodes: Vec<NodeId>,
        actions: Vec<String>,
    ) -> usize {
        let id = self.sets.len();
        for &node in &nodes {
            self.node_to_set.insert(node, id);
        }
        self.sets.push(InformationSet {
            id,
            player,
            nodes,
            actions,
        });
        id
    }

    /// Get the information set for a node.
    pub fn set_for_node(&self, node: NodeId) -> Option<&InformationSet> {
        self.node_to_set.get(&node).map(|&id| &self.sets[id])
    }

    /// Check if two nodes are in the same information set.
    pub fn same_set(&self, a: NodeId, b: NodeId) -> bool {
        match (self.node_to_set.get(&a), self.node_to_set.get(&b)) {
            (Some(sa), Some(sb)) => sa == sb,
            _ => false,
        }
    }

    /// Validate that an information structure is consistent with a game tree.
    ///
    /// Checks that:
    /// - All nodes in a set belong to the same player
    /// - All nodes in a set have the same number of actions
    /// - No node appears in multiple sets
    pub fn validate(&self, tree: &GameTree) -> Result<(), String> {
        for info_set in &self.sets {
            // Check all nodes exist and belong to same player
            let mut player = None;
            for &nid in &info_set.nodes {
                let node = tree.get_node(nid)
                    .ok_or_else(|| format!("Node {} not found", nid))?;
                match player {
                    None => player = Some(node.player()),
                    Some(p) => {
                        if node.player() != p {
                            return Err(format!(
                                "Info set {} has nodes with different players", info_set.id
                            ));
                        }
                    }
                }
            }

            // Check action counts match
            for &nid in &info_set.nodes {
                let node = tree.get_node(nid).unwrap();
                if node.actions.len() != info_set.actions.len() {
                    return Err(format!(
                        "Node {} has {} actions but info set has {}",
                        nid, node.actions.len(), info_set.actions.len()
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Build a simultaneous-move game as an extensive-form game with information sets.
///
/// Player 0 moves first, but player 1 cannot observe the move (they are in one info set).
pub fn simultaneous_game_info_sets(n_actions_p0: usize, n_actions_p1: usize) -> InformationStructure {
    let mut info = InformationStructure::new();
    // All P1 decision nodes go in one information set
    let p1_nodes: Vec<NodeId> = (1..=n_actions_p0).collect();
    let p1_actions: Vec<String> = (0..n_actions_p1).map(|i| format!("A{}", i)).collect();
    info.add_set(1, p1_nodes, p1_actions);
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::GameTree;

    #[test]
    fn test_info_set_creation() {
        let mut info = InformationStructure::new();
        let id = info.add_set(0, vec![0, 1], vec!["L".into(), "R".into()]);
        assert_eq!(id, 0);
        assert_eq!(info.sets.len(), 1);
    }

    #[test]
    fn test_same_set() {
        let mut info = InformationStructure::new();
        info.add_set(0, vec![0, 1, 2], vec!["L".into(), "R".into()]);
        assert!(info.same_set(0, 1));
        assert!(info.same_set(0, 2));
        assert!(!info.same_set(0, 3));
    }

    #[test]
    fn test_set_for_node() {
        let mut info = InformationStructure::new();
        info.add_set(0, vec![0, 1], vec!["L".into(), "R".into()]);
        let s = info.set_for_node(0).unwrap();
        assert_eq!(s.player, 0);
        assert!(info.set_for_node(99).is_none());
    }

    #[test]
    fn test_validate_consistent() {
        let mut tree = GameTree::new("Test");
        let root = tree.add_decision(0, "P0");
        let n0 = tree.add_decision(1, "P1-a");
        let n1 = tree.add_decision(1, "P1-b");
        let t0 = tree.add_terminal(vec![1.0, 2.0], "End0");
        let t1 = tree.add_terminal(vec![3.0, 4.0], "End1");
        let t2 = tree.add_terminal(vec![5.0, 6.0], "End2");
        let t3 = tree.add_terminal(vec![7.0, 8.0], "End3");
        tree.add_action(root, "L", n0);
        tree.add_action(root, "R", n1);
        tree.add_action(n0, "U", t0);
        tree.add_action(n0, "D", t1);
        tree.add_action(n1, "U", t2);
        tree.add_action(n1, "D", t3);

        let mut info = InformationStructure::new();
        info.add_set(1, vec![n0, n1], vec!["U".into(), "D".into()]);
        assert!(info.validate(&tree).is_ok());
    }
}
