// Subgraph Context Condenser: Multi-hop AST trace compression
// Implements: R-028 (Sub-1500 token high-fidelity context for LLM agents)

use crate::db::{DbLink, DbNode};
use serde::Serialize;
use std::collections::{HashSet, VecDeque};

#[derive(Serialize, Debug)]
pub struct CondensedContext {
    pub root_symbol: String,
    pub token_estimate: usize,
    pub formatted_markdown: String,
    pub direct_callers: Vec<String>,
    pub direct_callees: Vec<String>,
    pub related_files: Vec<String>,
}

pub struct ContextCondenser;

impl ContextCondenser {
    /// Condense a multi-hop neighborhood around a node or symbol into an agent-ready markdown block
    pub fn condense(
        target_symbol: &str,
        nodes: &[DbNode],
        links: &[DbLink],
        max_hops: usize,
    ) -> CondensedContext {
        let mut relevant_node_ids = HashSet::new();
        let mut direct_callers = Vec::new();
        let mut direct_callees = Vec::new();
        let mut related_files_set = HashSet::new();

        // 1. Locate root node(s) matching symbol or label
        let root_nodes: Vec<&DbNode> = nodes
            .iter()
            .filter(|n| n.label.eq_ignore_ascii_case(target_symbol) || n.id == target_symbol)
            .collect();

        let mut root_node_ids = HashSet::new();
        for root in &root_nodes {
            relevant_node_ids.insert(root.id.clone());
            root_node_ids.insert(root.id.clone());
            related_files_set.insert(root.file_path.clone());
        }

        // 2. BFS traversal up to max_hops (hard-capped at MAX_CONDENSED_NODES to prevent context blowout)
        const MAX_CONDENSED_NODES: usize = 60;
        let mut queue = VecDeque::new();
        for root in &root_nodes {
            queue.push_back((root.id.clone(), 0));
        }

        let mut truncated_count: usize = 0;
        while let Some((curr_id, depth)) = queue.pop_front() {
            if depth >= max_hops {
                continue;
            }

            for link in links {
                // Stop expanding if we've hit the node cap
                if relevant_node_ids.len() >= MAX_CONDENSED_NODES {
                    truncated_count += 1;
                    continue;
                }

                if link.source == curr_id {
                    // Outgoing: curr CALLS target
                    if relevant_node_ids.insert(link.target.clone()) {
                        queue.push_back((link.target.clone(), depth + 1));
                    }
                    if depth == 0 {
                        direct_callees.push(link.target.clone());
                    }
                } else if link.target == curr_id {
                    // Incoming: source CALLS curr
                    if relevant_node_ids.insert(link.source.clone()) {
                        queue.push_back((link.source.clone(), depth + 1));
                    }
                    if depth == 0 {
                        direct_callers.push(link.source.clone());
                    }
                }
            }
        }

        // 3. Format compact Markdown block with two-tier token budgeting (R-028)
        // Reserves at least 35% of token budget (~2100 chars) for Section 4 structural traces
        const MAX_OUTPUT_CHARS: usize = 6000;
        const MAX_SYMBOLS_CHARS: usize = 3800;

        let mut md = String::new();
        md.push_str(&format!("### 🧭 Omni-Graph AST Subgraph: `{}`\n\n", target_symbol));
        md.push_str("> High-fidelity deterministic AST slice (condensed for agent reasoning).\n\n");

        md.push_str("#### Identified Symbols & Signatures\n");
        let mut symbols_overflow = false;
        for n in nodes.iter().filter(|n| relevant_node_ids.contains(&n.id)) {
            related_files_set.insert(n.file_path.clone());

            let is_root = root_node_ids.contains(&n.id);
            let snippet = if is_root {
                n.text.trim().to_string()
            } else {
                let first_line = n.text.lines().next().unwrap_or("").trim();
                if first_line.len() > 140 {
                    format!("{}...", &first_line[..137])
                } else {
                    first_line.to_string()
                }
            };

            let entry = format!(
                "- **`{}`** ({}) in [`{}:{}`]({})\n  ```{}\n  {}\n  ```\n",
                n.label, n.kind, n.file_path, n.line_start, n.file_path, n.language, snippet
            );

            if md.len() + entry.len() > MAX_SYMBOLS_CHARS {
                symbols_overflow = true;
                break;
            }
            md.push_str(&entry);
        }

        if symbols_overflow {
            md.push_str("\n> ℹ️ Additional neighbor signatures condensed to preserve call trace budget.\n");
        }

        if truncated_count > 0 {
            md.push_str(&format!(
                "\n> ⚠️ Graph truncated: {} additional edges skipped (node cap: {}).\n",
                truncated_count, MAX_CONDENSED_NODES
            ));
        }

        md.push_str("\n#### Structural Relationships (Call / Import Traces)\n");
        for link in links.iter().filter(|l| relevant_node_ids.contains(&l.source) || relevant_node_ids.contains(&l.target)) {
            let edge_str = format!(
                "- `{}` ──[{}: {}]──▶ `{}`\n",
                link.source, link.category, link.edge_type, link.target
            );
            if md.len() + edge_str.len() > MAX_OUTPUT_CHARS {
                md.push_str(
                    "\n> ⚠️ Output truncated at ~1500 tokens. Use smaller HOPS or narrower workspace filter.\n"
                );
                break;
            }
            md.push_str(&edge_str);
        }

        let token_estimate = md.len() / 4; // Standard heuristic: 4 chars/token

        CondensedContext {
            root_symbol: target_symbol.to_string(),
            token_estimate,
            formatted_markdown: md,
            direct_callers,
            direct_callees,
            related_files: related_files_set.into_iter().collect(),
        }
    }
}
