// Tree-sitter AST Parser for multi-language semantic code extraction
// Implements: R-003 (Deterministic AST parsing)
// Extracts functions, structs, classes, imports, and calls

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use tree_sitter::{Node, Parser};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedNode {
    pub id: String,
    pub workspace: String,
    pub label: String,
    pub kind: String, // function, struct, class, interface, trait, import
    pub file_path: String,
    pub language: String,
    pub line_start: usize,
    pub line_end: usize,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtractedEdge {
    pub workspace: String,
    pub source_id: String,
    pub target_label: String,
    pub edge_type: String, // CALLS, IMPORTS, CONTAINS, IMPLEMENTS
    pub category: String,  // EXTRACTED
}

#[derive(Clone, Debug)]
pub struct ParseResult {
    pub nodes: Vec<ExtractedNode>,
    pub edges: Vec<ExtractedEdge>,
}

pub struct CodeParser;

impl CodeParser {
    fn safe_truncate(s: &str, max_chars: usize) -> &str {
        match s.char_indices().nth(max_chars) {
            Some((idx, _)) => &s[..idx],
            None => s,
        }
    }

    pub fn parse_file(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let extension = file_path.rsplit('.').next()?;
        if matches!(extension, "md" | "markdown" | "mdx") {
            return Self::parse_markdown(workspace, file_path, content);
        }
        if matches!(extension, "yaml" | "yml") {
            return Self::parse_yaml(workspace, file_path, content);
        }
        if extension == "json" {
            return Self::parse_json(workspace, file_path, content);
        }
        if matches!(extension, "sh" | "bash" | "zsh") {
            return Self::parse_shell(workspace, file_path, content);
        }
        if extension == "sql" {
            return Self::parse_sql(workspace, file_path, content);
        }
        if extension == "toml" {
            return Self::parse_toml(workspace, file_path, content);
        }
        let (language_name, mut parser) = match extension {
            "rs" => {
                let mut p = Parser::new();
                p.set_language(&tree_sitter_rust::language()).ok()?;
                ("rust", p)
            }
            "py" => {
                let mut p = Parser::new();
                p.set_language(&tree_sitter_python::language()).ok()?;
                ("python", p)
            }
            "go" => {
                let mut p = Parser::new();
                p.set_language(&tree_sitter_go::language()).ok()?;
                ("go", p)
            }
            "js" | "jsx" => {
                let mut p = Parser::new();
                p.set_language(&tree_sitter_javascript::language()).ok()?;
                ("javascript", p)
            }
            "ts" | "tsx" => {
                let mut p = Parser::new();
                p.set_language(&tree_sitter_typescript::language_typescript()).ok()?;
                ("typescript", p)
            }
            _ => return None,
        };

        let tree = parser.parse(content, None)?;
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        Self::traverse_node(
            tree.root_node(),
            content,
            workspace,
            file_path,
            language_name,
            None,
            &mut nodes,
            &mut edges,
        );

        Some(ParseResult { nodes, edges })
    }

    fn traverse_node(
        node: Node,
        content: &str,
        workspace: &str,
        file_path: &str,
        language: &str,
        current_parent_id: Option<&str>,
        nodes: &mut Vec<ExtractedNode>,
        edges: &mut Vec<ExtractedEdge>,
    ) {
        let kind = node.kind();
        let start_pos = node.start_position();
        let end_pos = node.end_position();

        let mut node_id: Option<String> = None;

        // Extract functions and methods
        if kind == "function_item"
            || kind == "function_definition"
            || kind == "function_declaration"
            || kind == "method_definition"
        {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &content[name_node.byte_range()];
                let snippet = &content[node.byte_range()];
                let text = Self::safe_truncate(snippet, 1000);

                let id = format!("{}:{}:{}:{}", workspace, file_path, name, start_pos.row + 1);
                nodes.push(ExtractedNode {
                    id: id.clone(),
                    workspace: workspace.to_string(),
                    label: name.to_string(),
                    kind: "function".to_string(),
                    file_path: file_path.to_string(),
                    language: language.to_string(),
                    line_start: start_pos.row + 1,
                    line_end: end_pos.row + 1,
                    text: text.to_string(),
                });

                if let Some(parent) = current_parent_id {
                    edges.push(ExtractedEdge {
                        workspace: workspace.to_string(),
                        source_id: parent.to_string(),
                        target_label: name.to_string(),
                        edge_type: "CONTAINS".to_string(),
                        category: "EXTRACTED".to_string(),
                    });
                }

                node_id = Some(id);
            }
        }
        // Extract structs, classes, and types
        else if kind == "struct_item"
            || kind == "class_definition"
            || kind == "class_declaration"
            || kind == "type_item"
        {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = &content[name_node.byte_range()];
                let snippet = &content[node.byte_range()];
                let text = Self::safe_truncate(snippet, 1000);

                let id = format!("{}:{}:{}:{}", workspace, file_path, name, start_pos.row + 1);
                nodes.push(ExtractedNode {
                    id: id.clone(),
                    workspace: workspace.to_string(),
                    label: name.to_string(),
                    kind: "struct".to_string(),
                    file_path: file_path.to_string(),
                    language: language.to_string(),
                    line_start: start_pos.row + 1,
                    line_end: end_pos.row + 1,
                    text: text.to_string(),
                });

                node_id = Some(id);
            }
        }
        // Extract function call expressions
        else if kind == "call_expression" {
            if let Some(caller_id) = current_parent_id {
                let callee_opt = node
                    .child_by_field_name("function")
                    .or_else(|| node.child(0));

                if let Some(callee_node) = callee_opt {
                    let callee_name = &content[callee_node.byte_range()];
                    // Clean up callee (e.g. self.foo -> foo, std::io::read -> read)
                    let clean_callee = callee_name
                        .rsplit('.')
                        .next()
                        .unwrap_or(callee_name)
                        .rsplit("::")
                        .next()
                        .unwrap_or(callee_name)
                        .trim();

                    if !clean_callee.is_empty() {
                        edges.push(ExtractedEdge {
                            workspace: workspace.to_string(),
                            source_id: caller_id.to_string(),
                            target_label: clean_callee.to_string(),
                            edge_type: "CALLS".to_string(),
                            category: "EXTRACTED".to_string(),
                        });
                    }
                }
            }
        }
        // Extract imports and uses
        else if kind == "use_declaration"
            || kind == "import_statement"
            || kind == "import_from_statement"
        {
            let snippet = &content[node.byte_range()];
            let id = format!("{}:{}:import:{}", workspace, file_path, start_pos.row + 1);
            nodes.push(ExtractedNode {
                id: id.clone(),
                workspace: workspace.to_string(),
                label: snippet.trim().to_string(),
                kind: "import".to_string(),
                file_path: file_path.to_string(),
                language: language.to_string(),
                line_start: start_pos.row + 1,
                line_end: end_pos.row + 1,
                text: snippet.trim().to_string(),
            });
        }

        // Recurse down children, passing down enclosing node id if present
        let next_parent = node_id.as_deref().or(current_parent_id);
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::traverse_node(
                child,
                content,
                workspace,
                file_path,
                language,
                next_parent,
                nodes,
                edges,
            );
        }
    }

    /// Semantic parser for Markdown documents (.md, .markdown)
    /// Extracts sections, chapters, concepts, and embedded code blocks with hierarchical CONTAINS edges
    pub fn parse_markdown(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut seen_edges: HashSet<(String, String, String)> = HashSet::new();

        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return None;
        }

        struct SectionMarker {
            level: usize,
            label: String,
            line_start: usize,
            body_lines: Vec<String>,
            node_id: String,
        }

        let mut current_section: Option<SectionMarker> = None;
        let mut section_stack: Vec<(usize, String, String)> = Vec::new(); // (level, label, node_id)

        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_lines: Vec<String> = Vec::new();
        let mut code_start_line = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            // Handle fenced code blocks
            if trimmed.starts_with("```") {
                if in_code_block {
                    // Ending code block
                    in_code_block = false;
                    let code_text = code_lines.join("\n");
                    let lang = if code_lang.is_empty() { "code" } else { &code_lang };

                    // Extract function or symbol definition name if present
                    let mut fn_name = None;
                    for cline in &code_lines {
                        let ctrim = cline.trim();
                        if let Some(rest) = ctrim.strip_prefix("func ") {
                            let name = rest.split(&['(', ' '][..]).next().unwrap_or("");
                            if !name.is_empty() { fn_name = Some(name.to_string()); break; }
                        } else if let Some(rest) = ctrim.strip_prefix("def ") {
                            let name = rest.split(&['(', ':', ' '][..]).next().unwrap_or("");
                            if !name.is_empty() { fn_name = Some(name.to_string()); break; }
                        } else if let Some(rest) = ctrim.strip_prefix("fn ") {
                            let name = rest.split(&['(', '<', ' '][..]).next().unwrap_or("");
                            if !name.is_empty() { fn_name = Some(name.to_string()); break; }
                        } else if let Some(rest) = ctrim.strip_prefix("class ") {
                            let name = rest.split(&['(', ':', '{', ' '][..]).next().unwrap_or("");
                            if !name.is_empty() { fn_name = Some(name.to_string()); break; }
                        } else if let Some(rest) = ctrim.strip_prefix("kind: ") {
                            fn_name = Some(format!("k8s:{}", rest.trim()));
                            break;
                        }
                    }

                    let parent_label = current_section.as_ref().map(|s| s.label.as_str()).unwrap_or("document");
                    let (label, kind) = match fn_name {
                        Some(name) => (name, "function".to_string()),
                        None => (format!("{} ({})", parent_label, lang), "snippet".to_string()),
                    };

                    let snippet_id = format!("{}:{}:{}:{}", workspace, file_path, label, code_start_line);
                    let snippet_text = Self::safe_truncate(&code_text, 1000).to_string();

                    nodes.push(ExtractedNode {
                        id: snippet_id.clone(),
                        workspace: workspace.to_string(),
                        label: label.clone(),
                        kind,
                        file_path: file_path.to_string(),
                        language: lang.to_string(),
                        line_start: code_start_line,
                        line_end: line_num,
                        text: snippet_text,
                    });

                    // Link parent section -> snippet
                    if let Some(cur) = &current_section {
                        edges.push(ExtractedEdge {
                            workspace: workspace.to_string(),
                            source_id: cur.node_id.clone(),
                            target_label: label.clone(),
                            edge_type: "CONTAINS".to_string(),
                            category: "EXTRACTED".to_string(),
                        });
                    }

                    code_lines.clear();
                    code_lang.clear();
                } else {
                    // Starting code block
                    in_code_block = true;
                    code_start_line = line_num;
                    code_lang = trimmed.trim_start_matches('`').trim().to_lowercase();
                    code_lines.clear();
                }
                continue;
            }

            if in_code_block {
                code_lines.push(line.to_string());
                continue;
            }

            // Check for Markdown headings: #, ##, ###, ####
            let is_heading = trimmed.starts_with('#');
            if is_heading {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                if level >= 1 && level <= 4 {
                    let heading_text = trimmed[level..].trim();
                    let clean_label = heading_text.trim_matches(&['*', '_', '`'][..]).trim().to_string();

                    if !clean_label.is_empty() {
                        // Flush previous section
                        if let Some(prev) = current_section.take() {
                            let text = Self::safe_truncate(&prev.body_lines.join("\n"), 1000).to_string();
                            nodes.push(ExtractedNode {
                                id: prev.node_id.clone(),
                                workspace: workspace.to_string(),
                                label: prev.label.clone(),
                                kind: if prev.level == 1 { "document".to_string() } else { "section".to_string() },
                                file_path: file_path.to_string(),
                                language: "markdown".to_string(),
                                line_start: prev.line_start,
                                line_end: line_num.saturating_sub(1),
                                text,
                            });
                        }

                        let node_id = format!("{}:{}:{}:{}", workspace, file_path, clean_label, line_num);

                        // Hierarchy edges: link to current parent in stack
                        while let Some((parent_level, _, _)) = section_stack.last() {
                            if *parent_level >= level {
                                section_stack.pop();
                            } else {
                                break;
                            }
                        }

                        if let Some((_, _, parent_id)) = section_stack.last() {
                            edges.push(ExtractedEdge {
                                workspace: workspace.to_string(),
                                source_id: parent_id.clone(),
                                target_label: clean_label.clone(),
                                edge_type: "CONTAINS".to_string(),
                                category: "EXTRACTED".to_string(),
                            });
                        }

                        section_stack.push((level, clean_label.clone(), node_id.clone()));
                        current_section = Some(SectionMarker {
                            level,
                            label: clean_label,
                            line_start: line_num,
                            body_lines: vec![format!("{} {}", "#".repeat(level), heading_text)],
                            node_id,
                        });
                        continue;
                    }
                }
            }

            // Normal content line
            if let Some(cur) = current_section.as_mut() {
                cur.body_lines.push(line.to_string());
            } else if !trimmed.is_empty() {
                // Content before first header
                let doc_label = file_path.rsplit('/').next().unwrap_or(file_path).to_string();
                let node_id = format!("{}:{}:{}:{}", workspace, file_path, doc_label, 1);
                current_section = Some(SectionMarker {
                    level: 1,
                    label: doc_label.clone(),
                    line_start: 1,
                    body_lines: vec![line.to_string()],
                    node_id: node_id.clone(),
                });
                section_stack.push((1, doc_label, node_id));
            }

            // Extract hyperlinks [text](target) and backtick references `ident`
            if let Some(cur) = &current_section {
                // 1. Hyperlinks: [text](target)
                let mut cursor = *line;
                while let Some(open_sq) = cursor.find('[') {
                    if let Some(close_sq) = cursor[open_sq..].find(']') {
                        let abs_close_sq = open_sq + close_sq;
                        if cursor[abs_close_sq..].starts_with("](") {
                            let target_start = abs_close_sq + 2;
                            if let Some(close_paren) = cursor[target_start..].find(')') {
                                let target = cursor[target_start..target_start + close_paren].trim();
                                let clean_target = target.split('#').next().unwrap_or(target).trim();
                                if clean_target.ends_with(".md")
                                    || clean_target.ends_with(".markdown")
                                    || clean_target.ends_with(".py")
                                    || clean_target.ends_with(".rs")
                                    || clean_target.ends_with(".go")
                                    || clean_target.ends_with(".ts")
                                    || clean_target.ends_with(".js")
                                    || clean_target.starts_with("./")
                                    || clean_target.starts_with("../")
                                {
                                    let target_label = clean_target.rsplit('/').next().unwrap_or(clean_target);
                                    if !target_label.is_empty() {
                                        let key = (cur.node_id.clone(), target_label.to_string(), "LINKS_TO".to_string());
                                        if seen_edges.insert(key) {
                                            edges.push(ExtractedEdge {
                                                workspace: workspace.to_string(),
                                                source_id: cur.node_id.clone(),
                                                target_label: target_label.to_string(),
                                                edge_type: "LINKS_TO".to_string(),
                                                category: "EXTRACTED".to_string(),
                                            });
                                        }
                                    }
                                }
                                cursor = &cursor[target_start + close_paren + 1..];
                                continue;
                            }
                        }
                    }
                    cursor = &cursor[open_sq + 1..];
                }

                // 2. Backtick code symbol references: `ident`
                let mut bt_start = None;
                for (i, c) in line.char_indices() {
                    if c == '`' {
                        if let Some(s) = bt_start {
                            let slice = line[s + 1..i].trim();
                            if slice.len() >= 3 && slice.len() <= 64
                                && slice.chars().all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.')
                                && slice.chars().next().map_or(false, |ch| ch.is_alphabetic() || ch == '_')
                            {
                                if !matches!(slice, "true" | "false" | "null" | "none" | "None" | "self" | "this"
                                    | "str" | "int" | "bool" | "def" | "class" | "fn" | "let" | "mut" | "const"
                                    | "return" | "git" | "npm" | "cargo" | "pip" | "http" | "https" | "node"
                                    | "type" | "array" | "float" | "list" | "dict")
                                {
                                    let key = (cur.node_id.clone(), slice.to_string(), "REFERENCES".to_string());
                                    if seen_edges.insert(key) {
                                        edges.push(ExtractedEdge {
                                            workspace: workspace.to_string(),
                                            source_id: cur.node_id.clone(),
                                            target_label: slice.to_string(),
                                            edge_type: "REFERENCES".to_string(),
                                            category: "EXTRACTED".to_string(),
                                        });
                                    }
                                }
                            }
                            bt_start = None;
                        } else {
                            bt_start = Some(i);
                        }
                    }
                }
            }
        }

        // Flush trailing section
        if let Some(last) = current_section {
            let text = Self::safe_truncate(&last.body_lines.join("\n"), 1000).to_string();
            nodes.push(ExtractedNode {
                id: last.node_id,
                workspace: workspace.to_string(),
                label: last.label,
                kind: if last.level == 1 { "document".to_string() } else { "section".to_string() },
                file_path: file_path.to_string(),
                language: "markdown".to_string(),
                line_start: last.line_start,
                line_end: lines.len(),
                text,
            });
        }

        if nodes.is_empty() {
            None
        } else {
            Some(ParseResult { nodes, edges })
        }
    }

    /// Semantic parser for YAML files (.yaml, .yml, Kubernetes manifests, Compose configs)
    pub fn parse_yaml(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path).file_name()?.to_string_lossy().to_string();
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "manifest".to_string(),
            file_path: file_path.to_string(),
            language: "yaml".to_string(),
            line_start: 1,
            line_end: content.lines().count().max(1),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        let mut doc_start = 1;
        let mut doc_lines: Vec<String> = Vec::new();
        let mut current_kind: Option<String> = None;
        let mut current_name: Option<String> = None;

        let flush_doc = |start: usize, end: usize, lines: &[String], kind: Option<String>, name: Option<String>, nodes: &mut Vec<ExtractedNode>, edges: &mut Vec<ExtractedEdge>| {
            if lines.is_empty() { return; }
            let k = kind.unwrap_or_else(|| "resource".to_string());
            let n = name.unwrap_or_else(|| format!("{}:L{}", filename, start));
            let res_id = format!("node:{}_{}_{}", workspace, file_path, n);
            let text = lines.join("\n");
            nodes.push(ExtractedNode {
                id: res_id.clone(),
                workspace: workspace.to_string(),
                label: n.clone(),
                kind: k,
                file_path: file_path.to_string(),
                language: "yaml".to_string(),
                line_start: start,
                line_end: end,
                text: Self::safe_truncate(&text, 1000).to_string(),
            });
            edges.push(ExtractedEdge {
                workspace: workspace.to_string(),
                source_id: file_root_id.clone(),
                target_label: n,
                edge_type: "CONTAINS".to_string(),
                category: "EXTRACTED".to_string(),
            });
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed == "---" {
                flush_doc(doc_start, line_num.saturating_sub(1), &doc_lines, current_kind.take(), current_name.take(), &mut nodes, &mut edges);
                doc_lines.clear();
                doc_start = line_num + 1;
                continue;
            }
            if trimmed.starts_with("kind:") {
                let val = trimmed.strip_prefix("kind:").unwrap_or("").trim().trim_matches(|c| c == '\'' || c == '"');
                if !val.is_empty() { current_kind = Some(val.to_string()); }
            } else if trimmed.starts_with("name:") && current_name.is_none() {
                let val = trimmed.strip_prefix("name:").unwrap_or("").trim().trim_matches(|c| c == '\'' || c == '"');
                if !val.is_empty() { current_name = Some(val.to_string()); }
            }
            doc_lines.push(line.to_string());
        }
        flush_doc(doc_start, content.lines().count().max(1), &doc_lines, current_kind, current_name, &mut nodes, &mut edges);

        Some(ParseResult { nodes, edges })
    }

    /// Semantic parser for JSON configuration files (.json)
    pub fn parse_json(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path).file_name()?.to_string_lossy().to_string();
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "config".to_string(),
            file_path: file_path.to_string(),
            language: "json".to_string(),
            line_start: 1,
            line_end: content.lines().count().max(1),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(obj) = val.as_object() {
                for (key, v) in obj {
                    let key_id = format!("node:{}_{}_{}", workspace, file_path, key);
                    let snippet = serde_json::to_string(v).unwrap_or_default();
                    nodes.push(ExtractedNode {
                        id: key_id,
                        workspace: workspace.to_string(),
                        label: format!("{}:{}", filename, key),
                        kind: "property".to_string(),
                        file_path: file_path.to_string(),
                        language: "json".to_string(),
                        line_start: 1,
                        line_end: content.lines().count().max(1),
                        text: Self::safe_truncate(&snippet, 1000).to_string(),
                    });
                    edges.push(ExtractedEdge {
                        workspace: workspace.to_string(),
                        source_id: file_root_id.clone(),
                        target_label: format!("{}:{}", filename, key),
                        edge_type: "CONTAINS".to_string(),
                        category: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        Some(ParseResult { nodes, edges })
    }

    /// Semantic parser for Shell scripts (.sh, .bash, .zsh)
    pub fn parse_shell(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path).file_name()?.to_string_lossy().to_string();
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "script".to_string(),
            file_path: file_path.to_string(),
            language: "bash".to_string(),
            line_start: 1,
            line_end: content.lines().count().max(1),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            let mut fn_name = None;

            if let Some(rest) = trimmed.strip_prefix("function ") {
                let name = rest.split(&['(', ' ', '{'][..]).next().unwrap_or("");
                if !name.is_empty() { fn_name = Some(name); }
            } else if trimmed.contains("()") && (trimmed.ends_with('{') || trimmed.contains("() {")) {
                let name = trimmed.split("()").next().unwrap_or("").trim();
                if !name.is_empty() && !name.contains(' ') { fn_name = Some(name); }
            }

            if let Some(name) = fn_name {
                let fn_id = format!("node:{}_{}_{}", workspace, file_path, name);
                let snippet_lines: Vec<&str> = lines[idx..idx + 25.min(lines.len() - idx)].to_vec();
                let snippet = snippet_lines.join("\n");
                nodes.push(ExtractedNode {
                    id: fn_id,
                    workspace: workspace.to_string(),
                    label: name.to_string(),
                    kind: "function".to_string(),
                    file_path: file_path.to_string(),
                    language: "bash".to_string(),
                    line_start: line_num,
                    line_end: (line_num + snippet_lines.len()).saturating_sub(1),
                    text: Self::safe_truncate(&snippet, 1000).to_string(),
                });
                edges.push(ExtractedEdge {
                    workspace: workspace.to_string(),
                    source_id: file_root_id.clone(),
                    target_label: name.to_string(),
                    edge_type: "CONTAINS".to_string(),
                    category: "EXTRACTED".to_string(),
                });
            }
        }

        Some(ParseResult { nodes, edges })
    }

    /// Semantic parser for SQL schemas and queries (.sql)
    pub fn parse_sql(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path).file_name()?.to_string_lossy().to_string();
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "sql".to_string(),
            file_path: file_path.to_string(),
            language: "sql".to_string(),
            line_start: 1,
            line_end: content.lines().count().max(1),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let upper = line.trim().to_uppercase();
            let mut obj_kind = None;
            let mut obj_name = None;

            if upper.starts_with("CREATE TABLE") {
                obj_kind = Some("table");
                let rest = line.trim()[12..].trim();
                let name = rest.trim_start_matches("IF NOT EXISTS").trim().split(&['(', ' '][..]).next().unwrap_or("").trim_matches('`').trim_matches('"');
                if !name.is_empty() { obj_name = Some(name); }
            } else if upper.starts_with("CREATE VIEW") {
                obj_kind = Some("view");
                let rest = line.trim()[11..].trim();
                let name = rest.trim_start_matches("IF NOT EXISTS").trim().split(&['(', ' ', 'A', 'S'][..]).next().unwrap_or("").trim_matches('`').trim_matches('"');
                if !name.is_empty() { obj_name = Some(name); }
            } else if upper.starts_with("CREATE PROCEDURE") || upper.starts_with("CREATE FUNCTION") {
                obj_kind = Some("function");
                let rest = line.trim()[16..].trim();
                let name = rest.trim_start_matches("OR REPLACE").trim().split(&['(', ' '][..]).next().unwrap_or("").trim_matches('`').trim_matches('"');
                if !name.is_empty() { obj_name = Some(name); }
            }

            if let (Some(k), Some(name)) = (obj_kind, obj_name) {
                let obj_id = format!("node:{}_{}_{}", workspace, file_path, name);
                let snippet_lines: Vec<&str> = lines[idx..idx + 30.min(lines.len() - idx)].to_vec();
                let snippet = snippet_lines.join("\n");
                nodes.push(ExtractedNode {
                    id: obj_id,
                    workspace: workspace.to_string(),
                    label: name.to_string(),
                    kind: k.to_string(),
                    file_path: file_path.to_string(),
                    language: "sql".to_string(),
                    line_start: line_num,
                    line_end: (line_num + snippet_lines.len()).saturating_sub(1),
                    text: Self::safe_truncate(&snippet, 1000).to_string(),
                });
                edges.push(ExtractedEdge {
                    workspace: workspace.to_string(),
                    source_id: file_root_id.clone(),
                    target_label: name.to_string(),
                    edge_type: "CONTAINS".to_string(),
                    category: "EXTRACTED".to_string(),
                });
            }
        }

        Some(ParseResult { nodes, edges })
    }

    /// Semantic parser for TOML configuration files (.toml)
    pub fn parse_toml(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path).file_name()?.to_string_lossy().to_string();
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "config".to_string(),
            file_path: file_path.to_string(),
            language: "toml".to_string(),
            line_start: 1,
            line_end: content.lines().count().max(1),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        let lines: Vec<&str> = content.lines().collect();
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed.trim_matches(|c| c == '[' || c == ']').trim();
                if !section_name.is_empty() {
                    let sec_id = format!("node:{}_{}_{}", workspace, file_path, section_name);
                    let snippet_lines: Vec<&str> = lines[idx..idx + 20.min(lines.len() - idx)].to_vec();
                    let snippet = snippet_lines.join("\n");
                    nodes.push(ExtractedNode {
                        id: sec_id,
                        workspace: workspace.to_string(),
                        label: format!("{}:[{}]", filename, section_name),
                        kind: "section".to_string(),
                        file_path: file_path.to_string(),
                        language: "toml".to_string(),
                        line_start: line_num,
                        line_end: (line_num + snippet_lines.len()).saturating_sub(1),
                        text: Self::safe_truncate(&snippet, 1000).to_string(),
                    });
                    edges.push(ExtractedEdge {
                        workspace: workspace.to_string(),
                        source_id: file_root_id.clone(),
                        target_label: format!("{}:[{}]", filename, section_name),
                        edge_type: "CONTAINS".to_string(),
                        category: "EXTRACTED".to_string(),
                    });
                }
            }
        }

        Some(ParseResult { nodes, edges })
    }

    /// Universal fallback chunker for plain text, extensionless files, or syntax fallbacks
    /// Guarantees that EVERY valid text file produces nodes and vector embeddings.
    pub fn parse_fallback(workspace: &str, file_path: &str, content: &str) -> Option<ParseResult> {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return None;
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let filename = Path::new(file_path)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| file_path.to_string());
        let file_root_id = format!("node:{}_{}", workspace, file_path);

        let ext = file_path.rsplit('.').next().unwrap_or("text");
        let language = match ext {
            "txt" | "text" => "text",
            "sh" | "bash" | "zsh" => "bash",
            "sql" => "sql",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "dockerfile" | "Dockerfile" => "dockerfile",
            "makefile" | "Makefile" => "makefile",
            other => other,
        };

        // File root node
        nodes.push(ExtractedNode {
            id: file_root_id.clone(),
            workspace: workspace.to_string(),
            label: filename.clone(),
            kind: "file".to_string(),
            file_path: file_path.to_string(),
            language: language.to_string(),
            line_start: 1,
            line_end: lines.len(),
            text: Self::safe_truncate(content, 1000).to_string(),
        });

        // Chunk lines if > 40 lines
        if lines.len() > 40 {
            let chunk_size = 40;
            let mut start = 0;
            while start < lines.len() {
                let end = (start + chunk_size).min(lines.len());
                let block_lines = &lines[start..end];
                let block_text = block_lines.join("\n");
                let block_label = format!("{}: L{}-L{}", filename, start + 1, end);
                let block_id = format!("node:{}_{}#L{}-L{}", workspace, file_path, start + 1, end);

                nodes.push(ExtractedNode {
                    id: block_id,
                    workspace: workspace.to_string(),
                    label: block_label.clone(),
                    kind: "block".to_string(),
                    file_path: file_path.to_string(),
                    language: language.to_string(),
                    line_start: start + 1,
                    line_end: end,
                    text: Self::safe_truncate(&block_text, 1000).to_string(),
                });

                edges.push(ExtractedEdge {
                    workspace: workspace.to_string(),
                    source_id: file_root_id.clone(),
                    target_label: block_label,
                    edge_type: "CONTAINS".to_string(),
                    category: "EXTRACTED".to_string(),
                });

                start += chunk_size;
            }
        }

        Some(ParseResult { nodes, edges })
    }
}
