//! Node types for game trees.

use std::fmt;

/// Unique node identifier.
pub type NodeId = usize;

/// Player identifier (0, 1, ...). Chance/terminal use special values.
pub type PlayerId = usize;

/// Payoff for a player at a terminal node.
pub type Payoff = f64;

/// The type of a game tree node.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeType {
    /// A decision node where a player chooses an action.
    Decision { player: PlayerId },
    /// A chance (nature) node with probability distribution over actions.
    Chance { probabilities: Vec<f64> },
    /// A terminal (leaf) node with payoffs for each player.
    Terminal { payoffs: Vec<Payoff> },
}

/// Data associated with a node in the game tree.
#[derive(Clone, Debug)]
pub struct NodeData {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// The type of this node.
    pub node_type: NodeType,
    /// Label for this node (e.g., description of the game state).
    pub label: String,
    /// Actions available at this node (labels).
    pub actions: Vec<String>,
    /// Children node IDs, one per action.
    pub children: Vec<NodeId>,
    /// Parent node ID (None for root).
    pub parent: Option<NodeId>,
    /// The action label that led to this node from its parent.
    pub incoming_action: Option<String>,
}

impl NodeData {
    /// Create a new decision node.
    pub fn decision(id: NodeId, player: PlayerId, label: &str) -> Self {
        Self {
            id,
            node_type: NodeType::Decision { player },
            label: label.to_string(),
            actions: Vec::new(),
            children: Vec::new(),
            parent: None,
            incoming_action: None,
        }
    }

    /// Create a chance node.
    pub fn chance(id: NodeId, probabilities: Vec<f64>, label: &str) -> Self {
        Self {
            id,
            node_type: NodeType::Chance { probabilities },
            label: label.to_string(),
            actions: Vec::new(),
            children: Vec::new(),
            parent: None,
            incoming_action: None,
        }
    }

    /// Create a terminal node.
    pub fn terminal(id: NodeId, payoffs: Vec<Payoff>, label: &str) -> Self {
        Self {
            id,
            node_type: NodeType::Terminal { payoffs },
            label: label.to_string(),
            actions: Vec::new(),
            children: Vec::new(),
            parent: None,
            incoming_action: None,
        }
    }

    /// Is this a terminal node?
    pub fn is_terminal(&self) -> bool {
        matches!(self.node_type, NodeType::Terminal { .. })
    }

    /// Is this a decision node?
    pub fn is_decision(&self) -> bool {
        matches!(self.node_type, NodeType::Decision { .. })
    }

    /// Get the player at this decision node (panics if not a decision node).
    pub fn player(&self) -> PlayerId {
        match &self.node_type {
            NodeType::Decision { player } => *player,
            _ => panic!("Not a decision node"),
        }
    }

    /// Get payoffs at this terminal node (panics if not terminal).
    pub fn payoffs(&self) -> &[Payoff] {
        match &self.node_type {
            NodeType::Terminal { payoffs } => payoffs,
            _ => panic!("Not a terminal node"),
        }
    }

    /// Get the number of actions at this node.
    pub fn num_actions(&self) -> usize {
        self.actions.len()
    }
}

impl fmt::Display for NodeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.node_type {
            NodeType::Decision { player } => {
                write!(f, "Node {} [P{}: {}]", self.id, player, self.label)
            }
            NodeType::Chance { .. } => {
                write!(f, "Node {} [Chance: {}]", self.id, self.label)
            }
            NodeType::Terminal { payoffs } => {
                write!(f, "Node {} [Terminal: {:?}]", self.id, payoffs)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_node() {
        let n = NodeData::decision(0, 0, "Root");
        assert!(n.is_decision());
        assert!(!n.is_terminal());
        assert_eq!(n.player(), 0);
    }

    #[test]
    fn test_terminal_node() {
        let n = NodeData::terminal(3, vec![2.0, 1.0], "End");
        assert!(n.is_terminal());
        assert_eq!(n.payoffs(), &[2.0, 1.0]);
    }

    #[test]
    fn test_chance_node() {
        let n = NodeData::chance(1, vec![0.5, 0.5], "Nature");
        assert!(matches!(n.node_type, NodeType::Chance { .. }));
    }

    #[test]
    fn test_node_display() {
        let n = NodeData::decision(0, 1, "Choose");
        let s = format!("{}", n);
        assert!(s.contains("P1"));
    }
}
