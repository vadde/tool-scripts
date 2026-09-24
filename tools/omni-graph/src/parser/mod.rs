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
}
