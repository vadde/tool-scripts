---
name: new-tool
description: Scaffold a new tool from the canonical template. Use when creating any new tool, script, plugin, or workflow in the repository.
---

# Skill: Scaffold a New Tool

## When to Use

Use this skill whenever you need to create a new tool in the `tools/` directory.
This ensures every tool starts with the correct structure, files, and metadata.

---

## Prerequisites

- Know the tool name (kebab-case, e.g., `json-validator`)
- Know the tool category (Math, GenAI, DevOps, Data, Utilities, etc.)
- Know the primary language (Python, Go, Node.js, Rust, Shell, etc.)
- Have a clear problem statement

---

## Step-by-Step Procedure

### Step 1: Scaffold from Template

```bash
make new-tool NAME=<tool-name>
```

Or manually:
```bash
cp -r tools/_template tools/<tool-name>
```

### Step 2: Create the Specification

Create `specs/catalog/<tool-name>.md` using the template:

```bash
cp specs/_templates/tool-spec.md specs/catalog/<tool-name>.md
```

Fill in ALL sections:
- [ ] Overview & Problem Statement
- [ ] Requirements (numbered: R-001, R-002, ...)
- [ ] Interface Contract (inputs, outputs, flags)
- [ ] Constraints (performance, security, compatibility)
- [ ] Dependencies
- [ ] Acceptance Criteria

### Step 3: Initialize STATUS.md

Edit `tools/<tool-name>/STATUS.md`:

```yaml
---
tool: <tool-name>
status: draft
version: 0.1.0
language: <python|go|node|rust|shell>
category: <Math|GenAI|DevOps|Data|Utilities>
created: YYYY-MM-DD
last_updated: YYYY-MM-DD
owner: "@vadde"
spec: ../../specs/catalog/<tool-name>.md
---
```

### Step 4: Set Up Language-Specific Files

Based on the language, create the appropriate config:

| Language | Files to Create |
|----------|----------------|
| Python | `pyproject.toml`, `src/__init__.py`, `src/main.py` |
| Go | `go.mod`, `src/main.go` |
| Node.js | `package.json`, `src/index.ts` |
| Rust | `Cargo.toml`, `src/main.rs` |
| Shell | `src/main.sh` (with proper shebang) |

### Step 5: Update the Tool's Makefile

Edit `tools/<tool-name>/Makefile` to use the correct language commands:

```makefile
# Python example
test:
	cd src && python -m pytest ../tests/ -v

lint:
	cd src && ruff check . && ruff format --check .

run:
	cd src && python main.py
```

### Step 6: Update the Tool Catalog

Add an entry to `tools/README.md`:

```markdown
| [tool-name](tool-name/) | Category | Language | `draft` | Brief description |
```

### Step 7: Create Initial Commit

```bash
git add tools/<tool-name> specs/catalog/<tool-name>.md
git commit -m "feat(<tool-name>): scaffold new tool

Created tool structure with spec, STATUS, and initial files.
SDLC Status: draft"
```

### Step 8: Verify

Run the validation script:
```bash
make validate-specs
```

---

## Post-Scaffold Checklist

- [ ] `tools/<tool-name>/` exists with all required files
- [ ] `specs/catalog/<tool-name>.md` exists with at minimum Overview filled in
- [ ] `tools/<tool-name>/STATUS.md` has `status: draft`
- [ ] `tools/README.md` catalog updated
- [ ] Initial commit made with conventional message
- [ ] Validation passes

---

## Common Mistakes to Avoid

1. **Skipping the spec** — Every tool needs a spec, even simple ones
2. **Wrong directory** — Tool goes in `tools/`, not `scripts/`
3. **Missing STATUS.md** — Required for SDLC tracking
4. **Forgetting catalog** — Tool must appear in `tools/README.md`
5. **Non-kebab-case name** — Use `my-tool` not `myTool` or `my_tool`
