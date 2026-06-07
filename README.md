# extensive-form

A pure-Rust library for **extensive-form games** with zero external dependencies.

## Features

- **Game trees** — Build arbitrary game trees with decision nodes, chance nodes, and terminal nodes
- **Backward induction** — Solve finite games via backward induction
- **Subgame perfect equilibrium (SPE)** — Find SPE via rollback
- **Information sets** — Model imperfect information with information sets
- **Sequential move analysis** — Full support for multi-stage games

## Modules

| Module | Description |
|---|---|
| `tree` | Game tree construction and traversal |
| `node` | Node types (decision, chance, terminal) |
| `information` | Information sets for imperfect information |
| `backward` | Backward induction solver |
| `subgame` | Subgame perfect equilibrium |

## Quick Start

```rust
use extensive_form::tree::GameTree;
use extensive_form::node::{NodeData, NodeType};
use extensive_form::backward::backward_induction;

// Build a simple Ultimatum game tree
let mut tree = GameTree::new("Ultimatum");
// ... add nodes and solve via backward induction
```

## License

MIT OR Apache-2.0
