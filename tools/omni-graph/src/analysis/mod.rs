// Graph RAG Analysis: Community Detection & Hybrid Graph-RAG Retrieval
// Implements: R-012 (Community Detection / Leiden clustering) and Graph-RAG Hybrid Synthesis

use crate::db::{DbClient, DbLink, DbNode};
use crate::embedder::EmbedderClient;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommunitySummary {
    pub id: i32,
    pub name: String,
    pub node_count: usize,
    pub top_symbols: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphRagResponse {
    pub query: String,
    pub macroscopic_summary: String,
    pub communities_involved: Vec<CommunitySummary>,
    pub seed_symbols: Vec<String>,
    pub expanded_subgraph: String,
    pub token_estimate: usize,
}

/// Deterministic lightweight pseudo-random number generator (Xorshift64)
/// Used for shuffling node evaluation order in LPA to eliminate deterministic traversal bias (Finding #5)
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
}

pub struct CommunityDetector;

impl CommunityDetector {
    /// Detect communities using iterative label propagation (high-speed O(E) modular clustering)
    /// Uses randomized node iteration order per round to prevent oscillation and deterministic bias (Finding #5)
    pub fn detect(nodes: &[DbNode], links: &[DbLink], max_iterations: usize) -> HashMap<String, i32> {
        let mut labels: HashMap<String, i32> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        // 1. Initialize each node with its own unique label
        for (i, node) in nodes.iter().enumerate() {
            labels.insert(node.id.clone(), i as i32);
            adj.insert(node.id.clone(), Vec::new());
        }

        // 2. Build undirected adjacency list
        for link in links {
            if adj.contains_key(&link.source) && adj.contains_key(&link.target) {
                adj.get_mut(&link.source).unwrap().push(link.target.clone());
                adj.get_mut(&link.target).unwrap().push(link.source.clone());
            }
        }

        // 3. Iterative label propagation with randomized node ordering (Finding #5)
        let mut rng = SimpleRng::new(0x4d595f5345454431); // Deterministic seed for reproducible testing
        let mut node_indices: Vec<usize> = (0..nodes.len()).collect();

        for _ in 0..max_iterations {
            let mut changed = false;
            rng.shuffle(&mut node_indices);
            for &idx in &node_indices {
                let node = &nodes[idx];
                let neighbors = match adj.get(&node.id) {
                    Some(n) if !n.is_empty() => n,
                    _ => continue,
                };

                // Count neighbor label frequencies
                let mut freq: HashMap<i32, usize> = HashMap::new();
                for neighbor_id in neighbors {
                    if let Some(lbl) = labels.get(neighbor_id) {
                        *freq.entry(*lbl).or_insert(0) += 1;
                    }
                }

                // Choose most frequent label
                if let Some((&best_label, _)) = freq.iter().max_by_key(|&(_, count)| count) {
                    if let Some(curr_label) = labels.get_mut(&node.id) {
                        if *curr_label != best_label {
                            *curr_label = best_label;
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // 4. Normalize labels to compact IDs: 0, 1, 2, ...
        let mut label_map: HashMap<i32, i32> = HashMap::new();
        let mut next_id = 0;
        let mut normalized = HashMap::new();

        for (node_id, raw_label) in labels {
            let compact_id = *label_map.entry(raw_label).or_insert_with(|| {
                let id = next_id;
                next_id += 1;
                id
            });
            normalized.insert(node_id, compact_id);
        }

        normalized
    }

    /// Summarize detected communities (macroscopic view)
    pub fn summarize(nodes: &[DbNode], assignments: &HashMap<String, i32>) -> Vec<CommunitySummary> {
        Self::summarize_filtered(nodes, assignments, 1)
    }

    /// Summarize detected communities with minimum node count filter to eliminate singleton noise (Finding #6)
    pub fn summarize_filtered(
        nodes: &[DbNode],
        assignments: &HashMap<String, i32>,
        min_size: usize,
    ) -> Vec<CommunitySummary> {
        let mut groups: HashMap<i32, Vec<&DbNode>> = HashMap::new();
        for node in nodes {
            if let Some(&cid) = assignments.get(&node.id) {
                groups.entry(cid).or_default().push(node);
            }
        }

        let mut summaries = Vec::new();
        for (cid, members) in groups {
            if members.len() < min_size {
                continue;
            }
            let mut files_set = HashSet::new();
            let mut top_symbols = Vec::new();

            for m in &members {
                files_set.insert(m.file_path.clone());
                if top_symbols.len() < 5 {
                    top_symbols.push(m.label.clone());
                }
            }

            // Derive representative name from most common directory or top symbol
            let sample_file = members.first().map(|m| m.file_path.as_str()).unwrap_or("unknown");
            let dir_name = sample_file
                .rsplit_once('/')
                .map(|(dir, _)| dir)
                .unwrap_or(sample_file);

            let comm_name = format!("Cluster #{}: {}", cid, dir_name);

            summaries.push(CommunitySummary {
                id: cid,
                name: comm_name,
                node_count: members.len(),
                top_symbols,
                files: files_set.into_iter().collect(),
            });
        }

        summaries.sort_by_key(|s| std::cmp::Reverse(s.node_count));
        summaries
    }
}

pub struct GraphRagEngine;

impl GraphRagEngine {
    /// Hybrid Graph-RAG Retrieval: Combines Micro Vector Search + Macro Community Summaries + AST Traces
    pub async fn query(
        db: &DbClient,
        embedder: &EmbedderClient,
        prompt: &str,
        top_k: usize,
        workspace: Option<&str>,
    ) -> Result<GraphRagResponse, String> {
        // 1. Microscopic Vector Search
        let query_vec = embedder.embed_single(prompt).await?;
        let vector_hits = db.search_vector(&query_vec, top_k, workspace).await?;

        // 2. Fetch Graph Topology
        let (all_nodes, all_links) = db.get_graph(workspace).await?;

        // 3. Collect Seed Symbols and relevant nodes
        let mut seed_symbols = Vec::new();
        let mut seed_node_ids = HashSet::new();
        let mut touched_communities = HashSet::new();

        for hit in &vector_hits {
            seed_symbols.push(hit.label.clone());
            seed_node_ids.insert(hit.id.clone());
            if let Some(c) = hit.community {
                touched_communities.insert(c);
            }
        }

        // 4. One-hop AST neighbor expansion
        let mut expanded_node_ids = seed_node_ids.clone();
        for link in &all_links {
            if seed_node_ids.contains(&link.source) {
                expanded_node_ids.insert(link.target.clone());
            } else if seed_node_ids.contains(&link.target) {
                expanded_node_ids.insert(link.source.clone());
            }
        }

        // 5. Build Subgraph Markdown
        let mut sub_md = String::new();
        sub_md.push_str("#### Microscopic AST Slice (Seeds & Direct Callers/Callees)\n");
        for n in all_nodes.iter().filter(|n| expanded_node_ids.contains(&n.id)) {
            sub_md.push_str(&format!(
                "- **`{}`** ({}) in `{}:{}`\n  ```{}\n  {}\n  ```\n",
                n.label, n.kind, n.file_path, n.line_start, n.language, n.text.trim()
            ));
        }

        sub_md.push_str("\n#### Direct Structural Relations\n");
        for link in all_links.iter().filter(|l| expanded_node_ids.contains(&l.source) && expanded_node_ids.contains(&l.target)) {
            sub_md.push_str(&format!(
                "- `{}` ──[{}: {}]──▶ `{}`\n",
                link.source, link.category, link.edge_type, link.target
            ));
        }

        // 6. Community summaries for touched clusters
        let mut comm_assignments = HashMap::new();
        for n in &all_nodes {
            if let Some(c) = n.community {
                comm_assignments.insert(n.id.clone(), c);
            }
        }
        let all_summaries = CommunityDetector::summarize(&all_nodes, &comm_assignments);
        let relevant_summaries: Vec<CommunitySummary> = all_summaries
            .into_iter()
            .filter(|s| touched_communities.contains(&s.id))
            .collect();

        let mut macro_summary = String::new();
        macro_summary.push_str(&format!(
            "Query matched {} high-confidence AST seeds across {} architectural community clusters.\n",
            vector_hits.len(),
            relevant_summaries.len()
        ));
        for comm in &relevant_summaries {
            macro_summary.push_str(&format!(
                "- **{}** ({} symbols): Top keys: {}\n",
                comm.name,
                comm.node_count,
                comm.top_symbols.join(", ")
            ));
        }

        let total_chars = sub_md.len() + macro_summary.len();
        let token_estimate = total_chars / 4;

        Ok(GraphRagResponse {
            query: prompt.to_string(),
            macroscopic_summary: macro_summary,
            communities_involved: relevant_summaries,
            seed_symbols,
            expanded_subgraph: sub_md,
            token_estimate,
        })
    }
}
