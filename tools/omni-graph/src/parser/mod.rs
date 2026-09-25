// Tree-sitter AST Parser for multi-language semantic code extraction
// Implements: R-003 (Deterministic AST parsing)
// Extracts functions, structs, classes, imports, and calls

use serde::{Deserialize, Serialize};
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
        if matches!(extension, "md" | "markdown") {
            return Self::parse_markdown(workspace, file_path, content);
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
}
