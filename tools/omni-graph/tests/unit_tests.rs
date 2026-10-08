// =============================================================================
// Omni-Graph Unit & Integration Test Suite
// Covers: parser, condenser, community detection, API normalization
// =============================================================================

mod parser_tests {
    use omni_graph::parser::CodeParser;

    #[test]
    fn parse_rust_function() {
        let content = r#"
fn hello_world(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn greet(user: &str) {
    let msg = hello_world(user);
    println!("{}", msg);
}
"#;
        let result = CodeParser::parse_file("test-ws", "src/main.rs", content);
        assert!(result.is_some(), "Parser should return Some for valid Rust");
        let pr = result.unwrap();

        assert_eq!(pr.nodes.len(), 2, "Should extract 2 functions");
        assert_eq!(pr.nodes[0].label, "hello_world");
        assert_eq!(pr.nodes[0].kind, "function");
        assert_eq!(pr.nodes[0].language, "rust");
        assert_eq!(pr.nodes[0].workspace, "test-ws");
        assert_eq!(pr.nodes[0].file_path, "src/main.rs");
        assert!(pr.nodes[0].line_start > 0);
        assert!(pr.nodes[0].line_end >= pr.nodes[0].line_start);
        assert_eq!(pr.nodes[1].label, "greet");

        // Should have a CALLS edge from greet -> hello_world
        let calls: Vec<_> = pr
            .edges
            .iter()
            .filter(|e| e.edge_type == "CALLS")
            .collect();
        assert!(
            !calls.is_empty(),
            "Should detect CALLS edges from greet to hello_world"
        );
        assert!(calls.iter().any(|e| e.target_label == "hello_world"));
    }

    #[test]
    fn parse_rust_struct() {
        let content = r#"
struct Config {
    host: String,
    port: u16,
}
"#;
        let result = CodeParser::parse_file("test-ws", "config.rs", content).unwrap();
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.nodes[0].label, "Config");
        assert_eq!(result.nodes[0].kind, "struct");
    }

    #[test]
    fn parse_rust_imports() {
        let content = r#"
use std::collections::HashMap;
use std::sync::Arc;

fn do_nothing() {}
"#;
        let result = CodeParser::parse_file("test-ws", "lib.rs", content).unwrap();
        let imports: Vec<_> = result.nodes.iter().filter(|n| n.kind == "import").collect();
        assert_eq!(imports.len(), 2, "Should extract 2 use declarations");
    }

    #[test]
    fn parse_python_function() {
        let content = r#"
def calculate_sum(a, b):
    return a + b

def main():
    result = calculate_sum(1, 2)
    print(result)
"#;
        let result = CodeParser::parse_file("py-ws", "calc.py", content);
        assert!(result.is_some(), "Parser should handle Python");
        let pr = result.unwrap();
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert_eq!(fns.len(), 2);
        assert_eq!(fns[0].label, "calculate_sum");
        assert_eq!(fns[0].language, "python");
        assert_eq!(fns[1].label, "main");
    }

    #[test]
    fn parse_go_function() {
        let content = r#"
package main

func Add(a int, b int) int {
    return a + b
}
"#;
        let result = CodeParser::parse_file("go-ws", "math.go", content);
        assert!(result.is_some(), "Parser should handle Go");
        let pr = result.unwrap();
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert!(!fns.is_empty());
        assert_eq!(fns[0].label, "Add");
        assert_eq!(fns[0].language, "go");
    }

    #[test]
    fn parse_go_full_grammar_methods_structs_constants_imports() {
        let content = r#"
package solver

import (
    "fmt"
    "math"
)

const (
    ActionFulfill = 1
    ActionPick    = 2
)

type OptimizedSolver struct {
    iterations int
}

type Evaluator interface {
    Eval(score float64) bool
}

func (s *OptimizedSolver) executeFulfill(orderId string) error {
    return nil
}

func (s OptimizedSolver) GetIterations() int {
    return s.iterations
}
"#;
        let result = CodeParser::parse_file("go-ws", "solver.go", content);
        assert!(result.is_some(), "Parser should handle full Go grammar");
        let pr = result.unwrap();

        // 1. Check constants
        let consts: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "constant").collect();
        assert_eq!(consts.len(), 2);
        assert!(consts.iter().any(|c| c.label == "ActionFulfill"));
        assert!(consts.iter().any(|c| c.label == "ActionPick"));

        // 2. Check struct and interface
        let structs: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "struct").collect();
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].label, "OptimizedSolver");

        let ifaces: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "interface").collect();
        assert_eq!(ifaces.len(), 1);
        assert_eq!(ifaces[0].label, "Evaluator");

        // 3. Check methods
        let methods: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "method").collect();
        assert_eq!(methods.len(), 3);
        assert!(methods.iter().any(|m| m.label == "executeFulfill"));
        assert!(methods.iter().any(|m| m.label == "GetIterations"));
        assert!(methods.iter().any(|m| m.label == "Eval"));

        // 4. Check imports
        let imports: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "import").collect();
        assert_eq!(imports.len(), 2);

        // 5. Check DECLARES edges from struct and interface to methods
        let struct_declares: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "DECLARES" && e.source_id == "go-ws:solver.go:OptimizedSolver").collect();
        assert_eq!(struct_declares.len(), 2);

        let iface_declares: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "DECLARES" && e.target_label == "Eval").collect();
        assert_eq!(iface_declares.len(), 1);
    }

    #[test]
    fn parse_javascript_function() {
        let content = r#"
function fetchData(url) {
    return fetch(url);
}
"#;
        let result = CodeParser::parse_file("js-ws", "api.js", content);
        assert!(result.is_some(), "Parser should handle JavaScript");
        let pr = result.unwrap();
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert!(fns.len() >= 1);
        assert_eq!(fns[0].label, "fetchData");
        assert_eq!(fns[0].language, "javascript");
    }

    #[test]
    fn parse_typescript_function() {
        let content = r#"
function greet(name: string): string {
    return `Hello, ${name}`;
}
"#;
        let result = CodeParser::parse_file("ts-ws", "hello.ts", content);
        assert!(result.is_some(), "Parser should handle TypeScript");
        let pr = result.unwrap();
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert!(fns.len() >= 1);
        assert_eq!(fns[0].label, "greet");
        assert_eq!(fns[0].language, "typescript");
    }

    #[test]
    fn parse_unsupported_extension_returns_none() {
        let result = CodeParser::parse_file("ws", "data.xyz", "some random data");
        assert!(result.is_none());
    }

    #[test]
    fn parse_empty_file() {
        let result = CodeParser::parse_file("ws", "empty.rs", "");
        assert!(result.is_some());
        assert!(result.unwrap().nodes.is_empty());
    }

    #[test]
    fn node_ids_contain_workspace_and_path() {
        let content = "fn foo() {}";
        let pr = CodeParser::parse_file("my-project", "src/lib.rs", content).unwrap();
        assert!(!pr.nodes.is_empty());
        let id = &pr.nodes[0].id;
        assert!(
            id.contains("my-project"),
            "Node ID should contain workspace"
        );
        assert!(
            id.contains("src/lib.rs"),
            "Node ID should contain file path"
        );
    }

    #[test]
    fn parse_typescript_arrow_functions_and_interfaces() {
        let content = r#"
export const loadWorkspaces = async () => {
    return fetch('/api/workspaces');
};

const handleFilter = (query: string) => {
    console.log(query);
};

export interface UserConfig {
    host: string;
    port: number;
}

export type UserID = string;

export enum TaskState {
    Pending,
    Active,
    Done,
}
"#;
        let pr = CodeParser::parse_file("ui-ws", "App.tsx", content).unwrap();

        // 1. Functions (arrow functions)
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert_eq!(fns.len(), 2);
        assert!(fns.iter().any(|f| f.label == "loadWorkspaces"));
        assert!(fns.iter().any(|f| f.label == "handleFilter"));

        // 2. Interface
        let ifaces: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "interface").collect();
        assert_eq!(ifaces.len(), 1);
        assert_eq!(ifaces[0].label, "UserConfig");

        // 3. Type alias
        let types: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "type").collect();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].label, "UserID");

        // 4. Enum
        let enums: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "enum").collect();
        assert_eq!(enums.len(), 1);
        assert_eq!(enums[0].label, "TaskState");
    }

    #[test]
    fn parse_rust_impl_and_enums() {
        let content = r#"
pub struct CodeParser;

pub enum DeltaKind {
    Create,
    Modify,
    Delete,
}

pub trait IngestionStrategy {
    fn ingest(&self);
}

impl CodeParser {
    pub fn parse_file(path: &str) -> bool {
        true
    }
}
"#;
        let pr = CodeParser::parse_file("rust-ws", "lib.rs", content).unwrap();

        // 1. Struct
        assert!(pr.nodes.iter().any(|n| n.label == "CodeParser" && n.kind == "struct"));

        // 2. Enum
        assert!(pr.nodes.iter().any(|n| n.label == "DeltaKind" && n.kind == "enum"));

        // 3. Trait
        assert!(pr.nodes.iter().any(|n| n.label == "IngestionStrategy" && n.kind == "trait"));

        // 4. Method
        assert!(pr.nodes.iter().any(|n| n.label == "parse_file" && n.kind == "function"));

        // 5. DECLARES edge linking struct to method inside impl
        let declares: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "DECLARES").collect();
        assert!(!declares.is_empty(), "impl block should emit DECLARES edge from struct to method");
        assert!(declares.iter().any(|e| e.source_id == "rust-ws:lib.rs:CodeParser" && e.target_label == "parse_file"));
    }

    #[test]
    fn parse_python_async_functions() {
        let content = r#"
async def fetch_remote_data(endpoint: str):
    return await http_get(endpoint)

def sync_process():
    pass
"#;
        let pr = CodeParser::parse_file("py-ws", "async_worker.py", content).unwrap();
        let fns: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "function").collect();
        assert_eq!(fns.len(), 2);
        assert!(fns.iter().any(|f| f.label == "fetch_remote_data"));
        assert!(fns.iter().any(|f| f.label == "sync_process"));
    }

    #[test]
    fn parse_go_interface_methods() {
        let content = r#"
package solver

type Solver interface {
    Solve(orders []int) error
    Reset()
}
"#;
        let pr = CodeParser::parse_file("go-ws", "solver.go", content).unwrap();

        // 1. Interface node
        assert!(pr.nodes.iter().any(|n| n.label == "Solver" && n.kind == "interface"));

        // 2. Interface methods (method_spec)
        let methods: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "method").collect();
        assert_eq!(methods.len(), 2);
        assert!(methods.iter().any(|m| m.label == "Solve"));
        assert!(methods.iter().any(|m| m.label == "Reset"));

        // 3. DECLARES edge linking interface to method
        let declares: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "DECLARES").collect();
        assert_eq!(declares.len(), 2);
        assert!(declares.iter().any(|e| e.target_label == "Solve"));
        assert!(declares.iter().any(|e| e.target_label == "Reset"));
    }

    #[test]
    fn parse_python_calls_and_methods() {
        let content = r#"
class Worker:
    def execute(self):
        self.run_task()
        process_data()
"#;
        let pr = CodeParser::parse_file("py-ws", "worker.py", content).unwrap();

        // Check CALLS edges emitted from Python calls
        let calls: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "CALLS").collect();
        assert!(!calls.is_empty(), "Python call expressions should emit CALLS edges");
        assert!(calls.iter().any(|c| c.target_label == "run_task"));
        assert!(calls.iter().any(|c| c.target_label == "process_data"));
    }

    #[test]
    fn parse_rust_method_calls() {
        let content = r#"
fn orchestrate(client: &DbClient) {
    client.query_sql("SELECT 1;");
    format_output();
}
"#;
        let pr = CodeParser::parse_file("rs-ws", "orch.rs", content).unwrap();

        // Check CALLS edges emitted from Rust method_call_expression and call_expression
        let calls: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "CALLS").collect();
        assert!(!calls.is_empty(), "Rust calls should emit CALLS edges");
        assert!(calls.iter().any(|c| c.target_label == "query_sql"));
        assert!(calls.iter().any(|c| c.target_label == "format_output"));
    }

    #[test]
    fn parse_import_edges() {
        let pr_rs = CodeParser::parse_file("rs-ws", "lib.rs", "use crate::db::DbClient;").unwrap();
        assert!(pr_rs.edges.iter().any(|e| e.edge_type == "IMPORTS" && e.target_label == "DbClient"));

        let pr_py = CodeParser::parse_file("py-ws", "main.py", "from solver import GreedySolver").unwrap();
        assert!(pr_py.edges.iter().any(|e| e.edge_type == "IMPORTS" && e.target_label == "GreedySolver"));
    }

    #[test]
    fn parse_rust_trait_implementation() {
        let content = r#"
pub struct CustomSolver;

pub trait Solvable {
    fn solve(&self);
}

impl Solvable for CustomSolver {
    fn solve(&self) {
        println!("solved");
    }
}
"#;
        let pr = CodeParser::parse_file("rs-ws", "solver.rs", content).unwrap();
        let impl_edges: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "IMPLEMENTS").collect();
        assert_eq!(impl_edges.len(), 1, "Must emit IMPLEMENTS edge for impl Trait for Struct");
        assert_eq!(impl_edges[0].target_label, "Solvable");
        assert!(impl_edges[0].source_id.ends_with("CustomSolver"));
    }

    #[test]
    fn parse_python_class_inheritance() {
        let content = r#"
class Animal:
    pass

class Dog(Animal):
    def bark(self):
        pass
"#;
        let pr = CodeParser::parse_file("py-ws", "models.py", content).unwrap();
        let extends_edges: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "EXTENDS").collect();
        assert_eq!(extends_edges.len(), 1, "Must emit EXTENDS edge for Python class inheritance");
        assert_eq!(extends_edges[0].target_label, "Animal");
    }

    #[test]
    fn parse_typescript_class_heritage() {
        let content = r#"
export interface Runnable {
    run(): void;
}

export class TaskRunner extends BaseRunner implements Runnable {
    run() {
        console.log("running");
    }
}
"#;
        let pr = CodeParser::parse_file("ts-ws", "runner.ts", content).unwrap();
        let extends_edges: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "EXTENDS").collect();
        assert_eq!(extends_edges.len(), 1, "Must emit EXTENDS edge for TS class");
        assert_eq!(extends_edges[0].target_label, "BaseRunner");

        let impl_edges: Vec<_> = pr.edges.iter().filter(|e| e.edge_type == "IMPLEMENTS").collect();
        assert_eq!(impl_edges.len(), 1, "Must emit IMPLEMENTS edge for TS class implements clause");
        assert_eq!(impl_edges[0].target_label, "Runnable");
    }

    #[test]
    fn parse_rust_macro_definition_and_invocation() {
        let content = r#"
macro_rules! my_telemetry {
    ($msg:expr) => {
        println!("{}", $msg);
    };
}

fn track() {
    my_telemetry!("ping");
}
"#;
        let pr = CodeParser::parse_file("rs-ws", "macro_test.rs", content).unwrap();
        assert!(pr.nodes.iter().any(|n| n.label == "my_telemetry" && n.kind == "macro"));
        assert!(pr.edges.iter().any(|e| e.edge_type == "CALLS" && e.target_label == "my_telemetry"));
    }

    #[test]
    fn text_truncation_at_1000_chars() {
        // Generate a function with a very long body
        let body = "let x = 1;\n".repeat(200); // ~2200 chars
        let content = format!("fn huge_fn() {{\n{}}}", body);
        let pr = CodeParser::parse_file("ws", "big.rs", &content).unwrap();
        assert!(!pr.nodes.is_empty());
        // Text should be truncated to ~1000 chars
        assert!(
            pr.nodes[0].text.len() <= 1100,
            "Text should be truncated around 1000 chars, got {}",
            pr.nodes[0].text.len()
        );
    }

    #[test]
    fn parse_markdown_sections_and_snippets() {
        let content = r#"# Concurrency Foundations

Some introductory text about concurrency.

## Goroutines

A goroutine is a lightweight thread.

```go
func doWork(id int) {
    println(id)
}
```

## Channels

Channels connect concurrent goroutines.
"#;
        let pr = CodeParser::parse_file("test-ws", "docs/concurrency.md", content).unwrap();

        // Check extracted nodes
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"Concurrency Foundations"), "Should extract root title");
        assert!(labels.contains(&"Goroutines"), "Should extract H2 Goroutines");
        assert!(labels.contains(&"Channels"), "Should extract H2 Channels");
        assert!(labels.contains(&"doWork"), "Should extract embedded Go function doWork");

        // Verify snippet has language 'go' and kind 'function'
        let do_work_node = pr.nodes.iter().find(|n| n.label == "doWork").unwrap();
        assert_eq!(do_work_node.language, "go");
        assert_eq!(do_work_node.kind, "function");

        // Verify hierarchy edges: Concurrency Foundations -> Goroutines -> doWork
        assert!(pr.edges.iter().any(|e| e.target_label == "Goroutines" && e.edge_type == "CONTAINS"));
        assert!(pr.edges.iter().any(|e| e.target_label == "doWork" && e.edge_type == "CONTAINS"));
    }

    #[test]
    fn parse_markdown_python_code_block() {
        let content = r#"# GenAI Foundations

## Attention Mechanism

```python
def scaled_dot_product_attention(q, k, v):
    return torch.matmul(q, k.transpose(-2, -1))
```
"#;
        let pr = CodeParser::parse_file("genai-ws", "notes/attention.md", content).unwrap();
        let fn_node = pr.nodes.iter().find(|n| n.label == "scaled_dot_product_attention");
        assert!(fn_node.is_some(), "Should extract python function from markdown block");
        assert_eq!(fn_node.unwrap().language, "python");
    }

    #[test]
    fn parse_markdown_links_and_references() {
        let content = r#"# Modified Binary Search

## Overview
See prerequisite in [Two Pointers](../02-two-pointers/theory.md) and related [Rotated Array](problem_02_rotated.md).
The algorithm calls `binary_search` and references `SearchRange.find_bound` to eliminate half the search space.
"#;
        let pr = CodeParser::parse_file("dsa-ws", "17-binary-search/theory.md", content).unwrap();

        // Check LINKS_TO edges
        let links: Vec<&str> = pr.edges.iter()
            .filter(|e| e.edge_type == "LINKS_TO")
            .map(|e| e.target_label.as_str())
            .collect();
        assert!(links.contains(&"theory.md"), "Should extract relative markdown link to theory.md");
        assert!(links.contains(&"problem_02_rotated.md"), "Should extract link to problem_02_rotated.md");

        // Check REFERENCES edges
        let refs: Vec<&str> = pr.edges.iter()
            .filter(|e| e.edge_type == "REFERENCES")
            .map(|e| e.target_label.as_str())
            .collect();
        assert!(refs.contains(&"binary_search"), "Should extract backticked symbol reference `binary_search`");
        assert!(refs.contains(&"SearchRange.find_bound"), "Should extract backticked symbol `SearchRange.find_bound`");
    }

    #[test]
    fn parse_yaml_kubernetes_manifest() {
        let content = r#"apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend-app
spec:
  replicas: 3
---
apiVersion: v1
kind: Service
metadata:
  name: frontend-svc
spec:
  ports:
    - port: 80
"#;
        let pr = CodeParser::parse_file("k8s-ws", "manifests/frontend.yaml", content).unwrap();
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"frontend-app"), "Should extract Deployment name");
        assert!(labels.contains(&"frontend-svc"), "Should extract Service name");

        let dep_node = pr.nodes.iter().find(|n| n.label == "frontend-app").unwrap();
        assert_eq!(dep_node.kind, "Deployment");
        assert_eq!(dep_node.language, "yaml");

        assert!(pr.edges.iter().any(|e| e.target_label == "frontend-app" && e.edge_type == "CONTAINS"));
        assert!(pr.edges.iter().any(|e| e.target_label == "frontend-svc" && e.edge_type == "CONTAINS"));
    }

    #[test]
    fn parse_json_config() {
        let content = r#"{
  "name": "omni-graph-ui",
  "version": "1.0.0",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build"
  }
}"#;
        let pr = CodeParser::parse_file("test-ws", "package.json", content).unwrap();
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.iter().any(|l| l.contains("scripts")));
        assert!(labels.iter().any(|l| l.contains("version")));
    }

    #[test]
    fn parse_shell_functions() {
        let content = r#"#!/usr/bin/env bash
function build_stack() {
    cargo build --release
}

run_tests() {
    cargo test
}
"#;
        let pr = CodeParser::parse_file("sh-ws", "scripts/build.sh", content).unwrap();
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"build_stack"));
        assert!(labels.contains(&"run_tests"));
        let fn_node = pr.nodes.iter().find(|n| n.label == "build_stack").unwrap();
        assert_eq!(fn_node.kind, "function");
        assert_eq!(fn_node.language, "bash");
    }

    #[test]
    fn parse_sql_tables_and_functions() {
        let content = r#"
CREATE TABLE users (
    id INT PRIMARY KEY,
    username VARCHAR(50)
);

CREATE FUNCTION get_user_count() RETURNS INT AS $$
BEGIN
    RETURN 42;
END;
$$ LANGUAGE plpgsql;
"#;
        let pr = CodeParser::parse_file("db-ws", "schema.sql", content).unwrap();
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.contains(&"users"));
        assert!(labels.contains(&"get_user_count"));
    }

    #[test]
    fn parse_toml_tables() {
        let content = r#"
[package]
name = "omni-graph"
version = "0.1.0"

[dependencies]
tokio = "1.0"
"#;
        let pr = CodeParser::parse_file("ws", "Cargo.toml", content).unwrap();
        let labels: Vec<&str> = pr.nodes.iter().map(|n| n.label.as_str()).collect();
        assert!(labels.iter().any(|l| l.contains("[package]")));
        assert!(labels.iter().any(|l| l.contains("[dependencies]")));
    }

    #[test]
    fn parse_fallback_chunker() {
        let lines: Vec<String> = (1..=100).map(|i| format!("Note line number {}", i)).collect();
        let content = lines.join("\n");
        let pr = CodeParser::parse_fallback("ws", "notes/study-guide.sample", &content).unwrap();

        assert!(!pr.nodes.is_empty());
        let root = &pr.nodes[0];
        assert_eq!(root.kind, "file");
        assert_eq!(root.label, "study-guide.sample");

        let blocks: Vec<_> = pr.nodes.iter().filter(|n| n.kind == "block").collect();
        assert!(!blocks.is_empty(), "Should generate chunk blocks for 100-line text");
        assert!(pr.edges.iter().any(|e| e.edge_type == "CONTAINS"));
    }
}

mod condenser_tests {
    use omni_graph::condenser::ContextCondenser;
    use omni_graph::db::{DbLink, DbNode};

    fn make_node(id: &str, label: &str, kind: &str, file_path: &str) -> DbNode {
        DbNode {
            id: id.to_string(),
            workspace: Some("test".to_string()),
            label: label.to_string(),
            kind: kind.to_string(),
            file_path: file_path.to_string(),
            language: "rust".to_string(),
            line_start: 1,
            line_end: 10,
            text: format!("fn {}() {{}}", label),
            community: None,
            file_hash: None,
        }
    }

    fn make_link(source: &str, target: &str, edge_type: &str) -> DbLink {
        DbLink {
            id: format!("edge:{}_{}", source, target),
            workspace: Some("test".to_string()),
            source: source.to_string(),
            target: target.to_string(),
            edge_type: edge_type.to_string(),
            category: "EXTRACTED".to_string(),
        }
    }

    #[test]
    fn condense_finds_root_symbol() {
        let nodes = vec![
            make_node("n1", "init_pool", "function", "src/db.rs"),
            make_node("n2", "other_fn", "function", "src/main.rs"),
        ];
        let links = vec![];
        let result = ContextCondenser::condense("init_pool", &nodes, &links, 2);

        assert_eq!(result.root_symbol, "init_pool");
        assert!(result.formatted_markdown.contains("init_pool"));
        assert!(result.related_files.contains(&"src/db.rs".to_string()));
    }

    #[test]
    fn condense_follows_edges_up_to_max_hops() {
        let nodes = vec![
            make_node("n1", "root_fn", "function", "src/a.rs"),
            make_node("n2", "caller_fn", "function", "src/b.rs"),
            make_node("n3", "callee_fn", "function", "src/c.rs"),
            make_node("n4", "distant_fn", "function", "src/d.rs"),
        ];
        let links = vec![
            make_link("n2", "n1", "CALLS"),  // caller -> root
            make_link("n1", "n3", "CALLS"),  // root -> callee
            make_link("n3", "n4", "CALLS"),  // callee -> distant (hop 2)
        ];

        // With hops=1, should NOT reach distant_fn
        let result_1 = ContextCondenser::condense("root_fn", &nodes, &links, 1);
        assert!(result_1.formatted_markdown.contains("caller_fn"));
        assert!(result_1.formatted_markdown.contains("callee_fn"));
        assert!(
            !result_1.formatted_markdown.contains("distant_fn"),
            "Should NOT include 2-hop distant_fn with max_hops=1"
        );

        // With hops=2, SHOULD reach distant_fn
        let result_2 = ContextCondenser::condense("root_fn", &nodes, &links, 2);
        assert!(
            result_2.formatted_markdown.contains("distant_fn"),
            "Should include 2-hop distant_fn with max_hops=2"
        );
    }

    #[test]
    fn condense_populates_callers_and_callees() {
        let nodes = vec![
            make_node("n1", "target", "function", "src/a.rs"),
            make_node("n2", "caller", "function", "src/b.rs"),
            make_node("n3", "callee", "function", "src/c.rs"),
        ];
        let links = vec![
            make_link("n2", "n1", "CALLS"), // caller -> target
            make_link("n1", "n3", "CALLS"), // target -> callee
        ];

        let result = ContextCondenser::condense("target", &nodes, &links, 2);
        assert!(result.direct_callers.contains(&"n2".to_string()));
        assert!(result.direct_callees.contains(&"n3".to_string()));
    }

    #[test]
    fn condense_nonexistent_symbol_produces_empty() {
        let nodes = vec![make_node("n1", "existing", "function", "src/a.rs")];
        let result = ContextCondenser::condense("nonexistent", &nodes, &[], 2);
        assert!(result.direct_callers.is_empty());
        assert!(result.direct_callees.is_empty());
        assert!(result.related_files.is_empty());
    }

    #[test]
    fn condense_token_estimate_is_reasonable() {
        let nodes = vec![
            make_node("n1", "foo", "function", "src/a.rs"),
            make_node("n2", "bar", "function", "src/b.rs"),
        ];
        let links = vec![make_link("n1", "n2", "CALLS")];
        let result = ContextCondenser::condense("foo", &nodes, &links, 2);

        // Token estimate = len / 4
        assert!(result.token_estimate > 0);
        assert_eq!(result.token_estimate, result.formatted_markdown.len() / 4);
    }
}

mod community_tests {
    use omni_graph::analysis::CommunityDetector;
    use omni_graph::db::{DbLink, DbNode};

    fn make_node(id: &str) -> DbNode {
        DbNode {
            id: id.to_string(),
            workspace: Some("test".to_string()),
            label: id.to_string(),
            kind: "function".to_string(),
            file_path: format!("src/{}.rs", id),
            language: "rust".to_string(),
            line_start: 1,
            line_end: 10,
            text: format!("fn {}() {{}}", id),
            community: None,
            file_hash: None,
        }
    }

    fn make_link(source: &str, target: &str) -> DbLink {
        DbLink {
            id: format!("edge:{}_{}", source, target),
            workspace: Some("test".to_string()),
            source: source.to_string(),
            target: target.to_string(),
            edge_type: "CALLS".to_string(),
            category: "EXTRACTED".to_string(),
        }
    }

    #[test]
    fn detect_assigns_all_nodes() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let links = vec![make_link("a", "b"), make_link("b", "c")];
        let assignments = CommunityDetector::detect(&nodes, &links, 15);

        assert_eq!(assignments.len(), 3, "Every node should have a community");
        assert!(assignments.contains_key("a"));
        assert!(assignments.contains_key("b"));
        assert!(assignments.contains_key("c"));
    }

    #[test]
    fn detect_connected_nodes_same_community() {
        // Fully connected triangle should converge to same community
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let links = vec![
            make_link("a", "b"),
            make_link("b", "c"),
            make_link("a", "c"),
        ];
        let assignments = CommunityDetector::detect(&nodes, &links, 15);

        assert_eq!(
            assignments["a"], assignments["b"],
            "Connected nodes a and b should share a community"
        );
        assert_eq!(
            assignments["b"], assignments["c"],
            "Connected nodes b and c should share a community"
        );
    }

    #[test]
    fn detect_disconnected_components_different_communities() {
        // Two disconnected pairs
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("x"),
            make_node("y"),
        ];
        let links = vec![
            make_link("a", "b"), // cluster 1
            make_link("x", "y"), // cluster 2
        ];
        let assignments = CommunityDetector::detect(&nodes, &links, 15);

        assert_eq!(assignments["a"], assignments["b"]);
        assert_eq!(assignments["x"], assignments["y"]);
        assert_ne!(
            assignments["a"], assignments["x"],
            "Disconnected components should have different communities"
        );
    }

    #[test]
    fn detect_compact_ids_start_at_zero() {
        let nodes = vec![make_node("a"), make_node("b")];
        let links = vec![make_link("a", "b")];
        let assignments = CommunityDetector::detect(&nodes, &links, 15);

        let min_id = *assignments.values().min().unwrap();
        assert_eq!(min_id, 0, "Compact IDs should start at 0");
    }

    #[test]
    fn detect_empty_graph() {
        let assignments = CommunityDetector::detect(&[], &[], 15);
        assert!(assignments.is_empty());
    }

    #[test]
    fn detect_isolated_nodes() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let links = vec![]; // no edges
        let assignments = CommunityDetector::detect(&nodes, &links, 15);
        assert_eq!(assignments.len(), 3);
        // Each isolated node keeps its own community
        let unique_communities: std::collections::HashSet<_> =
            assignments.values().collect();
        assert_eq!(
            unique_communities.len(),
            3,
            "Isolated nodes should each be in their own community"
        );
    }

    #[test]
    fn summarize_produces_correct_counts() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let mut assignments = std::collections::HashMap::new();
        assignments.insert("a".to_string(), 0);
        assignments.insert("b".to_string(), 0);
        assignments.insert("c".to_string(), 1);

        let summaries = CommunityDetector::summarize(&nodes, &assignments);
        assert_eq!(summaries.len(), 2);

        let comm0 = summaries.iter().find(|s| s.id == 0).unwrap();
        assert_eq!(comm0.node_count, 2);

        let comm1 = summaries.iter().find(|s| s.id == 1).unwrap();
        assert_eq!(comm1.node_count, 1);
    }

    #[test]
    fn summarize_sorted_by_size_descending() {
        let nodes = vec![
            make_node("a"),
            make_node("b"),
            make_node("c"),
            make_node("d"),
        ];
        let mut assignments = std::collections::HashMap::new();
        assignments.insert("a".to_string(), 0);
        assignments.insert("b".to_string(), 1);
        assignments.insert("c".to_string(), 1);
        assignments.insert("d".to_string(), 1);

        let summaries = CommunityDetector::summarize(&nodes, &assignments);
        assert!(
            summaries[0].node_count >= summaries[1].node_count,
            "Summaries should be sorted by node_count descending"
        );
    }
}

mod api_normalize_tests {
    use omni_graph::api::normalize_workspace;

    #[test]
    fn normalize_full_path() {
        let result = normalize_workspace(Some("/Users/aparv/projects/my-repo"));
        assert_eq!(result, Some("my-repo".to_string()));
    }

    #[test]
    fn normalize_trailing_slash() {
        let result = normalize_workspace(Some("/Users/aparv/projects/my-repo/"));
        assert_eq!(result, Some("my-repo".to_string()));
    }

    #[test]
    fn normalize_bare_name() {
        let result = normalize_workspace(Some("session-explorer"));
        assert_eq!(result, Some("session-explorer".to_string()));
    }

    #[test]
    fn normalize_empty_string() {
        let result = normalize_workspace(Some(""));
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_whitespace_only() {
        let result = normalize_workspace(Some("   "));
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_none() {
        let result = normalize_workspace(None);
        assert_eq!(result, None);
    }

    #[test]
    fn normalize_single_segment_with_trailing_slash() {
        let result = normalize_workspace(Some("myproject/"));
        assert_eq!(result, Some("myproject".to_string()));
    }

    #[test]
    fn normalize_deeply_nested_path() {
        let result = normalize_workspace(Some("/a/b/c/d/e/deep-project"));
        assert_eq!(result, Some("deep-project".to_string()));
    }
}

mod escape_tests {
    use omni_graph::db::surql_escape;

    #[test]
    fn escape_single_quotes() {
        assert_eq!(surql_escape("hello'world"), "hello\\'world");
        assert_eq!(surql_escape("'test'"), "\\'test\\'");
    }

    #[test]
    fn escape_backslashes() {
        assert_eq!(surql_escape("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn escape_newlines_and_returns() {
        assert_eq!(surql_escape("line1\nline2\r\nline3"), "line1 line2 line3");
    }

    #[test]
    fn escape_null_and_control_chars() {
        assert_eq!(surql_escape("clean\0text\x07bell"), "cleantextbell");
    }

    #[test]
    fn defeat_sql_injection_payload() {
        let malicious = "' OR '1'='1' --; DROP TABLE node;";
        let escaped = surql_escape(malicious);
        assert_eq!(escaped, "\\' OR \\'1\\'=\\'1\\' --; DROP TABLE node;");
        assert!(escaped.starts_with("\\'"));
    }

    #[test]
    fn clean_record_id_variants() {
        use omni_graph::db::clean_record_id;
        assert_eq!(clean_record_id("node:`ws:file:func:10`"), "ws:file:func:10");
        assert_eq!(clean_record_id("node:⟨ws:file:func:10⟩"), "ws:file:func:10");
        assert_eq!(clean_record_id("node:ws:file:func:10"), "ws:file:func:10");
        assert_eq!(clean_record_id("`ws:file:func:10`"), "ws:file:func:10");
        assert_eq!(clean_record_id("⟨ws:file:func:10⟩"), "ws:file:func:10");
        assert_eq!(clean_record_id("ws:file:func:10"), "ws:file:func:10");
    }
}

mod lpa_rng_tests {
    use omni_graph::analysis::SimpleRng;

    #[test]
    fn rng_deterministic_shuffle() {
        let mut rng1 = SimpleRng::new(42);
        let mut rng2 = SimpleRng::new(42);
        let mut v1 = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let mut v2 = vec![1, 2, 3, 4, 5, 6, 7, 8];

        rng1.shuffle(&mut v1);
        rng2.shuffle(&mut v2);

        assert_eq!(v1, v2, "Same seed must produce identical shuffle");
        assert_eq!(v1.len(), 8);
        for item in 1..=8 {
            assert!(v1.contains(&item));
        }
    }

    #[test]
    fn rng_different_seeds_differ() {
        let mut rng1 = SimpleRng::new(12345);
        let mut rng2 = SimpleRng::new(67890);
        let mut v1: Vec<usize> = (0..20).collect();
        let mut v2: Vec<usize> = (0..20).collect();

        rng1.shuffle(&mut v1);
        rng2.shuffle(&mut v2);

        assert_ne!(v1, v2, "Different seeds should produce different shuffles");
    }
}

mod cluster_filter_tests {
    use omni_graph::analysis::CommunityDetector;
    use omni_graph::db::DbNode;
    use std::collections::HashMap;

    fn make_node(id: &str, file: &str, label: &str) -> DbNode {
        DbNode {
            id: id.to_string(),
            workspace: Some("test".to_string()),
            label: label.to_string(),
            kind: "function".to_string(),
            file_path: file.to_string(),
            language: "rust".to_string(),
            line_start: 1,
            line_end: 10,
            text: format!("fn {}() {{}}", label),
            community: None,
            file_hash: None,
        }
    }

    #[test]
    fn filter_singletons() {
        let nodes = vec![
            make_node("1", "src/a.rs", "fn_a1"),
            make_node("2", "src/a.rs", "fn_a2"),
            make_node("3", "src/a.rs", "fn_a3"),
            make_node("4", "src/b.rs", "fn_b1"), // singleton
        ];

        let mut assignments = HashMap::new();
        assignments.insert("1".to_string(), 0);
        assignments.insert("2".to_string(), 0);
        assignments.insert("3".to_string(), 0);
        assignments.insert("4".to_string(), 1); // singleton cluster

        // summarize without filter returns both
        let all = CommunityDetector::summarize(&nodes, &assignments);
        assert_eq!(all.len(), 2);

        // summarize_filtered with min_size=2 drops the singleton
        let filtered = CommunityDetector::summarize_filtered(&nodes, &assignments, 2);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 0);
        assert_eq!(filtered[0].node_count, 3);
    }
}

mod analytics_tests {
    use omni_graph::analytics::AnalyticsEngine;
    use std::fs::{self, File};
    use std::io::Write;
    use std::sync::Mutex;

    static BRAIN_ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_analytics_scan_and_detail_with_synthetic_session() {
        let _guard = BRAIN_ENV_MUTEX.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("omni_test_brain_{}", std::process::id()));
        let session_id = "test-session-12345";
        let session_dir = temp_dir.join(session_id).join(".system_generated").join("logs");
        fs::create_dir_all(&session_dir).expect("create temp session dir");

        let transcript_path = session_dir.join("transcript.jsonl");
        let mut file = File::create(&transcript_path).expect("create transcript");

        let step1 = serde_json::json!({
            "step_index": 0,
            "source": "USER_INPUT",
            "type": "USER_INPUT",
            "status": "DONE",
            "created_at": "2026-09-25T12:00:00Z",
            "content": "Look up symbol NewSessionIndex in GoLang",
        });

        let step2 = serde_json::json!({
            "step_index": 1,
            "source": "MODEL",
            "type": "PLANNER_RESPONSE",
            "status": "DONE",
            "created_at": "2026-09-25T12:00:05Z",
            "content": "Using omni-graph to find symbol definition",
            "thinking": "I should query the Omni-Graph AST rather than dumping whole files.",
            "tool_calls": [
                {
                    "name": "run_command",
                    "args": {
                        "CommandLine": "make graph-symbol SYM=NewSessionIndex"
                    }
                },
                {
                    "name": "view_file",
                    "args": {
                        "AbsolutePath": "/Users/aparv/code/server.go"
                    }
                }
            ]
        });

        writeln!(file, "{}", step1).unwrap();
        writeln!(file, "{}", step2).unwrap();
        drop(file);

        // Point BRAIN_DIR to temp_dir
        std::env::set_var("BRAIN_DIR", &temp_dir);

        let analytics = AnalyticsEngine::scan_analytics();
        assert_eq!(analytics.summary.total_sessions, 1);
        assert_eq!(analytics.summary.total_steps, 2);
        assert_eq!(analytics.summary.total_tool_calls, 2);
        assert_eq!(analytics.summary.total_omni_calls, 1);
        assert_eq!(analytics.summary.total_lsp_lookups, 1);
        assert!(analytics.summary.estimated_tokens_saved > 0);

        // Verify tools breakdown
        assert!(analytics.tools_breakdown.iter().any(|t| t.name == "run_command"));
        assert!(analytics.tools_breakdown.iter().any(|t| t.name == "view_file"));

        // Verify language telemetry detected .go
        assert!(analytics.languages_telemetry.iter().any(|l| l.language == "Go"));

        // Verify session detail
        let detail = AnalyticsEngine::get_session_detail(session_id);
        assert!(detail.is_some());
        let d = detail.unwrap();
        assert_eq!(d.session_id, session_id);
        assert_eq!(d.total_steps, 2);
        assert_eq!(d.total_tools, 2);
        assert_eq!(d.omni_tools, 1);
        assert_eq!(d.messages.len(), 2);
        assert_eq!(d.messages[1].tool_calls.len(), 2);
        assert!(d.messages[1].tool_calls[0].is_omni);
        assert_eq!(
            d.messages[1].tool_calls[0].omni_category,
            Some("Omni-Graph: AST Definition (LSP)".to_string())
        );

        // Non-existent session returns None
        assert!(AnalyticsEngine::get_session_detail("non-existent-session-xyz").is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
        std::env::remove_var("BRAIN_DIR");
    }

    #[test]
    fn test_dynamic_workspace_selection_dominance() {
        let _guard = BRAIN_ENV_MUTEX.lock().unwrap();
        let temp_dir = std::env::temp_dir().join(format!("omni_test_dominance_{}", std::process::id()));
        let session_id = "test-session-quarkdock-dom";
        let session_dir = temp_dir.join(session_id).join(".system_generated").join("logs");
        fs::create_dir_all(&session_dir).expect("create temp session dir");

        let transcript_path = session_dir.join("transcript.jsonl");
        let mut file = File::create(&transcript_path).expect("create transcript");

        // Initial prompt mentions QuarkDock
        let step0 = serde_json::json!({
            "step_index": 0,
            "source": "USER_INPUT",
            "type": "USER_INPUT",
            "status": "DONE",
            "created_at": "2026-09-29T12:00:00Z",
            "content": "welcome to QuarkDock code repo. we will be building frontend and backend",
        });

        // Step 1 touches tool-scripts once (e.g. reading rules)
        let step1 = serde_json::json!({
            "step_index": 1,
            "source": "MODEL",
            "type": "PLANNER_RESPONSE",
            "status": "DONE",
            "created_at": "2026-09-29T12:00:05Z",
            "tool_calls": [
                {
                    "name": "view_file",
                    "args": {
                        "AbsolutePath": "/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/tool-scripts/.agents/AGENTS.md"
                    }
                }
            ]
        });

        // Step 2 touches QuarkDock multiple times
        let step2 = serde_json::json!({
            "step_index": 2,
            "source": "MODEL",
            "type": "PLANNER_RESPONSE",
            "status": "DONE",
            "created_at": "2026-09-29T12:00:10Z",
            "tool_calls": [
                {
                    "name": "run_command",
                    "args": {
                        "Cwd": "/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/QuarkDock",
                        "CommandLine": "docker compose up -d"
                    }
                },
                {
                    "name": "view_file",
                    "args": {
                        "AbsolutePath": "/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/QuarkDock/services/ui/src/App.tsx"
                    }
                },
                {
                    "name": "write_to_file",
                    "args": {
                        "TargetFile": "/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/QuarkDock/services/api/main.py"
                    }
                }
            ]
        });

        writeln!(file, "{}", step0).unwrap();
        writeln!(file, "{}", step1).unwrap();
        writeln!(file, "{}", step2).unwrap();
        drop(file);

        std::env::set_var("BRAIN_DIR", &temp_dir);

        let analytics = AnalyticsEngine::scan_analytics();
        assert_eq!(analytics.summary.total_sessions, 1);
        let sess = &analytics.sessions[0];
        assert_eq!(
            sess.workspace,
            "QuarkDock",
            "QuarkDock must dominate over tool-scripts because it has 3 tool operations vs 1"
        );

        let detail = AnalyticsEngine::get_session_detail(session_id).expect("must find session detail");
        assert_eq!(
            detail.workspace,
            "QuarkDock",
            "Session detail must also resolve to QuarkDock"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
        std::env::remove_var("BRAIN_DIR");
    }
}

