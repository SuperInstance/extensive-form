//! Game tree construction and traversal.

use crate::node::{NodeData, NodeId, Payoff, PlayerId};
use std::collections::HashMap;

/// A game tree representing an extensive-form game.
#[derive(Clone, Debug)]
pub struct GameTree {
    /// Name of the game.
    pub name: String,
    /// All nodes indexed by their ID.
    pub nodes: HashMap<NodeId, NodeData>,
    /// The root node ID.
    pub root: Option<NodeId>,
    /// Next available node ID.
    next_id: NodeId,
    /// Number of players.
    pub num_players: usize,
}

impl GameTree {
    /// Create a new empty game tree.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
            num_players: 2,
        }
    }

    /// Create a game tree with a specified number of players.
    pub fn with_players(name: &str, num_players: usize) -> Self {
        Self {
            name: name.to_string(),
            nodes: HashMap::new(),
            root: None,
            next_id: 0,
            num_players,
        }
    }

    /// Allocate a new node ID.
    fn alloc_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Add a decision node and return its ID.
    pub fn add_decision(&mut self, player: PlayerId, label: &str) -> NodeId {
        let id = self.alloc_id();
        let node = NodeData::decision(id, player, label);
        self.nodes.insert(id, node);
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    /// Add a terminal node with payoffs.
    pub fn add_terminal(&mut self, payoffs: Vec<Payoff>, label: &str) -> NodeId {
        let id = self.alloc_id();
        let node = NodeData::terminal(id, payoffs, label);
        self.nodes.insert(id, node);
        id
    }

    /// Add a chance node.
    pub fn add_chance(&mut self, probabilities: Vec<f64>, label: &str) -> NodeId {
        let id = self.alloc_id();
        let node = NodeData::chance(id, probabilities, label);
        self.nodes.insert(id, node);
        if self.root.is_none() {
            self.root = Some(id);
        }
        id
    }

    /// Add an action from parent to child with a label.
    pub fn add_action(&mut self, parent: NodeId, action: &str, child: NodeId) {
        if let Some(node) = self.nodes.get_mut(&parent) {
            node.actions.push(action.to_string());
            node.children.push(child);
        }
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
            child_node.incoming_action = Some(action.to_string());
        }
    }

    /// Get a node by ID.
    pub fn get_node(&self, id: NodeId) -> Option<&NodeData> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID.
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(&id)
    }

    /// Get all terminal nodes.
    pub fn terminals(&self) -> Vec<&NodeData> {
        self.nodes.values().filter(|n| n.is_terminal()).collect()
    }

    /// Get the depth of a node (distance from root).
    pub fn depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = id;
        while let Some(node) = self.nodes.get(&current) {
            match node.parent {
                Some(p) => {
                    depth += 1;
                    current = p;
                }
                None => break,
            }
        }
        depth
    }

    /// Get the path from root to a given node.
    pub fn path_to(&self, id: NodeId) -> Vec<NodeId> {
        let mut path = vec![id];
        let mut current = id;
        while let Some(node) = self.nodes.get(&current) {
            match node.parent {
                Some(p) => {
                    path.push(p);
                    current = p;
                }
                None => break,
            }
        }
        path.reverse();
        path
    }

    /// Count total nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Build an Ultimatum game: P1 proposes split, P2 accepts/rejects.
    pub fn ultimatum_game(total: Payoff) -> Self {
        let mut tree = Self::new("Ultimatum Game");
        tree.num_players = 2;

        // P1 chooses how much to offer: Low or High
        let root = tree.add_decision(0, "P1 Proposes");
        let high = total * 0.8;
        let low = total * 0.2;

        // P2 responses for high offer
        let p2_high = tree.add_decision(1, "P2 Responds to High");
        let t_ha = tree.add_terminal(vec![total - high, high], "High Accepted");
        let t_hr = tree.add_terminal(vec![0.0, 0.0], "High Rejected");

        tree.add_action(p2_high, "Accept", t_ha);
        tree.add_action(p2_high, "Reject", t_hr);

        // P2 responses for low offer
        let p2_low = tree.add_decision(1, "P2 Responds to Low");
        let t_la = tree.add_terminal(vec![total - low, low], "Low Accepted");
        let t_lr = tree.add_terminal(vec![0.0, 0.0], "Low Rejected");

        tree.add_action(p2_low, "Accept", t_la);
        tree.add_action(p2_low, "Reject", t_lr);

        tree.add_action(root, "High Offer", p2_high);
        tree.add_action(root, "Low Offer", p2_low);

        tree
    }

    /// Build a Centipede game with `n` rounds.
    pub fn centipede_game(n: usize, pot: Payoff, growth: Payoff) -> Self {
        let mut tree = Self::new("Centipede Game");
        tree.num_players = 2;

        let root = tree.add_decision(0, "P1 Turn 1");
        Self::build_centipede(&mut tree, root, n, pot, growth, 0);
        tree
    }

    fn build_centipede(
        tree: &mut Self,
        current: NodeId,
        rounds_left: usize,
        pot: Payoff,
        growth: Payoff,
        round: usize,
    ) {
        let player = round % 2;
        let take_share = if player == 0 { 0.7 } else { 0.3 };
        let pass_pot = pot + growth;

        // Take action
        let t_take = tree.add_terminal(
            vec![pot * take_share, pot * (1.0 - take_share)],
            &format!("P{} Takes", player),
        );
        tree.add_action(current, "Take", t_take);

        // Pass action
        if rounds_left > 1 {
            let next = tree.add_decision(1 - player, &format!("P{} Turn {}", 1 - player, round + 2));
            tree.add_action(current, "Pass", next);
            Self::build_centipede(tree, next, rounds_left - 1, pass_pot, growth, round + 1);
        } else {
            // Final round: forced split
            let t_end = tree.add_terminal(
                vec![pass_pot * 0.5, pass_pot * 0.5],
                "Game Ends Cooperatively",
            );
            tree.add_action(current, "Pass", t_end);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = GameTree::new("Empty");
        assert_eq!(tree.node_count(), 0);
        assert!(tree.root.is_none());
    }

    #[test]
    fn test_simple_tree() {
        let mut tree = GameTree::new("Simple");
        let root = tree.add_decision(0, "Root");
        let left = tree.add_terminal(vec![1.0, 2.0], "Left");
        let right = tree.add_terminal(vec![3.0, 0.0], "Right");
        tree.add_action(root, "L", left);
        tree.add_action(root, "R", right);
        assert_eq!(tree.node_count(), 3);
        assert_eq!(tree.root, Some(root));
    }

    #[test]
    fn test_ultimatum_game() {
        let tree = GameTree::ultimatum_game(10.0);
        assert_eq!(tree.node_count(), 7);
        assert_eq!(tree.terminals().len(), 4);
    }

    #[test]
    fn test_centipede_game() {
        let tree = GameTree::centipede_game(3, 1.0, 1.0);
        assert!(tree.node_count() > 3);
        assert!(tree.terminals().len() >= 3);
    }

    #[test]
    fn test_depth() {
        let mut tree = GameTree::new("Depth");
        let root = tree.add_decision(0, "Root");
        let child = tree.add_terminal(vec![1.0], "Child");
        tree.add_action(root, "Go", child);
        assert_eq!(tree.depth(root), 0);
        assert_eq!(tree.depth(child), 1);
    }

    #[test]
    fn test_path_to() {
        let mut tree = GameTree::new("Path");
        let root = tree.add_decision(0, "Root");
        let mid = tree.add_decision(1, "Mid");
        let leaf = tree.add_terminal(vec![1.0, 2.0], "Leaf");
        tree.add_action(root, "Go", mid);
        tree.add_action(mid, "Go", leaf);
        let path = tree.path_to(leaf);
        assert_eq!(path.len(), 3);
    }
}
