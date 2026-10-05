// Graph RAG Analysis: Community Detection & Hybrid Graph-RAG Retrieval
// Implements: R-012 (Community Detection / Leiden clustering) and Graph-RAG Hybrid Synthesis

use crate::db::{DbClient, DbLink, DbNode, GalaxyRecord};
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

/// Assign weights to edges based on architectural significance
pub fn get_edge_weight(edge_type: &str) -> f32 {
    match edge_type {
        "IMPLEMENTS" => 3.0,
        "CALLS" => 2.0,
        "TYPE_REF" => 1.5,
        "IMPORTS" | "CONTAINS" => 1.0,
        _ => 1.0,
    }
}

pub struct CommunityDetector;

impl CommunityDetector {
    /// Detect communities using iterative label propagation (high-speed O(E) modular clustering)
    pub fn detect(nodes: &[DbNode], links: &[DbLink], max_iterations: usize) -> HashMap<String, i32> {
        Self::detect_with_seeds(nodes, links, max_iterations, None)
    }

    /// Detect communities with warm-seeded initial assignments to prevent cluster ID thrashing across live edits
    pub fn detect_with_seeds(
        nodes: &[DbNode],
        links: &[DbLink],
        max_iterations: usize,
        seeds: Option<&HashMap<String, i32>>,
    ) -> HashMap<String, i32> {
        let mut labels: HashMap<String, i32> = HashMap::new();
        let mut adj: HashMap<String, Vec<(String, f32)>> = HashMap::new();

        // 1. Initialize node labels from seeds if available, otherwise allocate unique IDs
        for (i, node) in nodes.iter().enumerate() {
            let initial_label = seeds
                .and_then(|s| s.get(&node.id).copied())
                .unwrap_or(i as i32 + 100_000);
            labels.insert(node.id.clone(), initial_label);
            adj.insert(node.id.clone(), Vec::new());
        }

        // 2. Build weighted undirected adjacency list
        for link in links {
            if adj.contains_key(&link.source) && adj.contains_key(&link.target) {
                let weight = get_edge_weight(&link.edge_type);
                adj.get_mut(&link.source).unwrap().push((link.target.clone(), weight));
                adj.get_mut(&link.target).unwrap().push((link.source.clone(), weight));
            }
        }

        // 3. Iterative label propagation with randomized node ordering
        let mut rng = SimpleRng::new(0x4d595f5345454431);
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

                // Accumulate weighted label frequencies
                let mut freq: HashMap<i32, f32> = HashMap::new();
                for (neighbor_id, weight) in neighbors {
                    if let Some(lbl) = labels.get(neighbor_id) {
                        *freq.entry(*lbl).or_insert(0.0) += *weight;
                    }
                }

                // Select label with maximum accumulated edge weight
                if let Some((&best_label, _)) = freq.iter().max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal)) {
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

        // 4. Normalize labels to compact IDs while preserving established seed IDs
        let mut label_map: HashMap<i32, i32> = HashMap::new();
        let mut next_id = 0;

        // Reserve existing seed IDs first
        if let Some(seed_map) = seeds {
            for &seed_id in seed_map.values() {
                if let std::collections::hash_map::Entry::Vacant(e) = label_map.entry(seed_id) {
                    e.insert(seed_id);
                    if seed_id >= next_id {
                        next_id = seed_id + 1;
                    }
                }
            }
        }

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

    /// Compute full architectural metrics (Ca, Ce, Instability, roles) and generate galaxy records
    pub fn compute_galaxy_metrics(
        workspace: &str,
        nodes: &[DbNode],
        links: &[DbLink],
        assignments: &HashMap<String, i32>,
    ) -> (Vec<CommunitySummary>, Vec<GalaxyRecord>) {
        let mut groups: HashMap<i32, Vec<&DbNode>> = HashMap::new();
        for node in nodes {
            if let Some(&cid) = assignments.get(&node.id) {
                groups.entry(cid).or_default().push(node);
            }
        }

        // Count internal and cross-boundary edges per community
        let mut internal_edges_map: HashMap<i32, usize> = HashMap::new();
        let mut afferent_coupling_map: HashMap<i32, usize> = HashMap::new(); // Ca: foreign -> this
        let mut efferent_coupling_map: HashMap<i32, usize> = HashMap::new(); // Ce: this -> foreign

        for link in links {
            let src_comm = assignments.get(&link.source);
            let tgt_comm = assignments.get(&link.target);

            if let (Some(&src_c), Some(&tgt_c)) = (src_comm, tgt_comm) {
                if src_c == tgt_c {
                    *internal_edges_map.entry(src_c).or_insert(0) += 1;
                } else {
                    *efferent_coupling_map.entry(src_c).or_insert(0) += 1;
                    *afferent_coupling_map.entry(tgt_c).or_insert(0) += 1;
                }
            }
        }

        let mut summaries = Vec::new();
        let mut galaxy_records = Vec::new();

        for (cid, members) in groups {
            let mut files_set = HashSet::new();
            let mut top_symbols = Vec::new();
            let mut languages = HashSet::new();
            let mut dir_counts: HashMap<String, usize> = HashMap::new();

            for m in &members {
                files_set.insert(m.file_path.clone());
                if !m.language.is_empty() {
                    languages.insert(m.language.clone());
                }
                if top_symbols.len() < 8 {
                    top_symbols.push(m.label.clone());
                }

                if let Some(parent) = std::path::Path::new(&m.file_path).parent() {
                    let p = parent.to_string_lossy().to_string();
                    if !p.is_empty() && p != "." {
                        *dir_counts.entry(p).or_insert(0) += 1;
                    }
                }
            }

            let dominant = dir_counts
                .into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(d, _)| d)
                .unwrap_or_else(|| {
                    members.first().map(|m| m.file_path.clone()).unwrap_or_else(|| "root".to_string())
                });

            let name = if dominant.is_empty() || dominant == "." {
                format!("Galaxy #{}", cid)
            } else {
                dominant.clone()
            };

            let internal_edges = internal_edges_map.get(&cid).copied().unwrap_or(0);
            let ca = afferent_coupling_map.get(&cid).copied().unwrap_or(0);
            let ce = efferent_coupling_map.get(&cid).copied().unwrap_or(0);
            let total_cut = ca + ce;

            let instability = if total_cut == 0 {
                0.5
            } else {
                (ce as f64) / (total_cut as f64)
            };

            let role = if instability <= 0.25 {
                "Core Foundation".to_string()
            } else if instability <= 0.65 {
                "Domain Service".to_string()
            } else {
                "Orchestrator / Leaf".to_string()
            };

            let langs_vec: Vec<String> = languages.into_iter().collect();

            summaries.push(CommunitySummary {
                id: cid,
                name: format!("Galaxy #{}: {}", cid, name),
                node_count: members.len(),
                top_symbols: top_symbols.clone(),
                files: files_set.into_iter().collect(),
            });

            galaxy_records.push(GalaxyRecord {
                workspace: workspace.to_string(),
                galaxy_id: cid,
                name: name.clone(),
                dominant_path: dominant,
                node_count: members.len(),
                internal_edges,
                external_edges: total_cut,
                afferent_coupling: ca,
                efferent_coupling: ce,
                instability,
                role,
                key_symbols: top_symbols,
                languages: langs_vec,
                updated_at: None,
            });
        }

        summaries.sort_by_key(|s| std::cmp::Reverse(s.node_count));
        galaxy_records.sort_by_key(|g| std::cmp::Reverse(g.node_count));

        (summaries, galaxy_records)
    }

    /// Infer community for a newly created AST node based on modal connection to existing graph
    #[allow(dead_code)]
    pub fn infer_neighbor_community(
        node_id: &str,
        all_links: &[DbLink],
        node_communities: &HashMap<String, i32>,
    ) -> Option<i32> {
        let mut votes: HashMap<i32, f32> = HashMap::new();
        for l in all_links {
            if l.source == node_id {
                if let Some(&c) = node_communities.get(&l.target) {
                    *votes.entry(c).or_insert(0.0) += get_edge_weight(&l.edge_type);
                }
            } else if l.target == node_id {
                if let Some(&c) = node_communities.get(&l.source) {
                    *votes.entry(c).or_insert(0.0) += get_edge_weight(&l.edge_type);
                }
            }
        }

        votes
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(c, _)| c)
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
