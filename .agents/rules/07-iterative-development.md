# Rule 07 — Iterative Development & Session Continuity

> **Priority**: 🔴 Critical — Every session must leave the tool in a state where
> ANY agent can pick it up immediately.

---

## The Continuity Manifesto

1. **Sessions Are Ephemeral, Tools Are Permanent** — Your context dies when the
   session ends; the tool's context files must capture everything you learned
2. **CONTEXT.md Is the Contract** — It is the single source of truth for a tool's
   current state, and it MUST be updated at the end of every session
3. **DEVLOG.md Is the Memory** — It is the append-only journal that records what
   happened, why, and what to do next
4. **Specs Are Living Documents** — Requirement statuses (⬜→🔨→✅→🧪) must be
   updated as implementation progresses
5. **No Orphan Work** — If you wrote code, it must be reflected in CONTEXT.md,
   DEVLOG.md, and the spec

---

## Session Start Protocol

**MANDATORY** — Before ANY action on a tool, complete this sequence:

```
┌─────────────────────────────────────────────────────────────┐
│                  SESSION START PROTOCOL                       │
│                                                               │
│  Step 1: Read tools/<name>/CONTEXT.md                        │
│          → Understand current state, progress, blockers       │
│          → Know what the LAST session accomplished            │
│          → Know what the NEXT STEPS are                       │
│                                                               │
│  Step 2: Read tools/<name>/DEVLOG.md (latest entry)          │
│          → Get detailed context from previous session(s)      │
│          → Understand recent decisions and rationale           │
│                                                               │
│  Step 3: Verify tools/<name>/STATUS.md                       │
│          → Confirm SDLC phase                                 │
│          → Check traceability matrix for gaps                 │
│          → Review blockers                                    │
│                                                               │
│  Step 4: Read specs/catalog/<name>.md (if implementing)      │
│          → Check which R-XXX requirements are pending          │
│          → Understand the interface contract                   │
│                                                               │
│  Step 5: Plan your session                                    │
│          → List specific requirements you'll address           │
│          → Estimate scope (don't over-commit)                 │
│                                                               │
│  ✅ Now you may begin work                                    │
└─────────────────────────────────────────────────────────────┘
```

### Shortcut for Continuing Work

If you're continuing in the same session (no context loss), you may skip to
Step 3 and verify nothing has changed.

---

## Session End Protocol

**MANDATORY** — Before ending ANY session that touched a tool, complete ALL steps:

```
┌─────────────────────────────────────────────────────────────┐
│                   SESSION END PROTOCOL                        │
│                                                               │
│  Step 1: Update spec requirement statuses                     │
│          → Mark implemented requirements: ⬜ → ✅              │
│          → Mark in-progress requirements: ⬜ → 🔨              │
│          → Mark tested requirements: ✅ → 🧪                  │
│                                                               │
│  Step 2: Update tools/<name>/CONTEXT.md                      │
│          → Update "Current State" section                      │
│          → Update "Progress" table with counts                │
│          → Update "Next Steps" for the next session           │
│          → Update "Active Blockers" if any                    │
│          → Update "Key Decisions" if any were made            │
│          → Update "Traceability Summary"                       │
│          → Set last_session date in YAML frontmatter          │
│                                                               │
│  Step 3: Append to tools/<name>/DEVLOG.md                    │
│          → Use the session entry template                      │
│          → Record: what was done, files changed,              │
│            decisions, blockers, next steps                    │
│          → Be SPECIFIC — file names, requirement IDs          │
│                                                               │
│  Step 4: Update tools/<name>/STATUS.md                       │
│          → Update implementation progress table               │
│          → Update traceability matrix rows                    │
│          → Advance SDLC phase if warranted                    │
│          → Add phase history entry if phase changed           │
│                                                               │
│  Step 5: Update tools/<name>/CHANGELOG.md (if releasing)     │
│          → Add entries under [Unreleased] section             │
│                                                               │
│  ✅ Session complete — tool is ready for any agent            │
└─────────────────────────────────────────────────────────────┘
```

---

## Spec Requirement Status Lifecycle

Every requirement in the spec MUST have a tracked status:

```
⬜ Not Started ──→ 🔨 In Progress ──→ ✅ Implemented ──→ 🧪 Tested
                                              │
                                              ↓
                                        ❌ Blocked/Deferred
```

### Status Definitions

| Symbol | Meaning | When to Use |
|--------|---------|-------------|
| `⬜` | Not started | Requirement exists but no code written |
| `🔨` | In progress | Code is being written but not complete |
| `✅` | Implemented | Code is complete, may need testing |
| `🧪` | Tested & verified | Code has passing tests, acceptance criteria met |
| `❌` | Blocked/deferred | Cannot proceed due to dependency or decision |

### Rules

1. **Never skip statuses** — A requirement must go ⬜→🔨→✅→🧪 in order
2. **Update in real-time** — Don't batch updates; update as you go
3. **Reference in code** — Every function implementing R-XXX must have a comment:
   ```go
   // Implements: R-001 (Session discovery)
   ```
4. **Reference in tests** — Every test for R-XXX must reference it:
   ```go
   func TestR001_SessionDiscovery(t *testing.T) { ... }
   ```

---

## CONTEXT.md Structure

The CONTEXT.md file has a strict structure. All sections are mandatory:

| Section | Purpose | Updated When |
|---------|---------|--------------|
| YAML Frontmatter | Machine-readable status, dates, blockers | Every session |
| Current State | Human-readable state summary | Every session |
| Progress | Quantitative tracking table | Every session |
| Quick Links | Navigation to all tool files | On scaffold, rarely changed |
| Next Steps | What to do when you pick this up | Every session end |
| Active Blockers | What's preventing progress | When blockers change |
| Key Decisions | Important choices and rationale | When decisions are made |
| Traceability Summary | Quick R-XXX status overview | When requirements change |

---

## DEVLOG.md Structure

The DEVLOG.md is append-only. New entries go at the **top** (reverse chronological).

### Required Entry Fields

Every session entry MUST include:

| Field | Purpose | Example |
|-------|---------|---------|
| Date | When | `2026-09-18` |
| Title | What (brief) | `Implemented JSONL parser and search engine` |
| Agent/Author | Who | `@antigravity-claude-opus` or `@vadde` |
| SDLC Phase | Lifecycle tracking | `spec-review` → `in-progress` |
| What Was Done | Deliverables | Bullet list of completed items |
| Requirements Addressed | Traceability | R-XXX with status changes |
| Files Changed | Audit trail | File paths with descriptions |
| Decisions Made | Institutional memory | Decision + rationale |
| Blockers | Risk tracking | Issues encountered |
| Next Steps | Continuity | Specific actions for next session |

---

## Multi-Tool Swarm Rules

When working on a monorepo with many tools:

### Tool Isolation

1. **Each tool is independent** — Changes to tool-A must NEVER break tool-B
2. **Each tool has its own context** — CONTEXT.md, DEVLOG.md, STATUS.md, spec
3. **Cross-tool dependencies are explicit** — Listed in spec Section 5.2
4. **Shared code goes in `scripts/`** — Not in any single tool

### Tool Discovery

When asked to work on a tool:
1. Check `tools/README.md` catalog for the tool
2. If it exists → read its CONTEXT.md
3. If it doesn't exist → use the `new-tool` skill to scaffold it

### Catalog Integrity

After ANY of these actions, run `make catalog && make status`:
- Creating a new tool
- Changing a tool's SDLC status
- Changing a tool's language or category
- Releasing a tool version

---

## Anti-Patterns (NEVER Do These)

| Anti-Pattern | Why It's Wrong | Do This Instead |
|-------------|----------------|-----------------
| Skip CONTEXT.md update | Next session starts blind | Always update at session end |
| Skip DEVLOG.md entry | Context is permanently lost | Always append an entry |
| Leave specs with all ⬜ | Can't track progress | Update statuses as you implement |
| Modify DEVLOG.md history | Destroys audit trail | Append-only; add corrections as new entries |
| Orphan code (no R-XXX ref) | Can't trace to requirements | Every function references a requirement |
| Over-commit in one session | Leaves messy state | Scope to what you can finish and document |
| Update CONTEXT.md mid-session | Confusing state | Update ONCE at session end |

---

## Emergency Recovery

If you find a tool in a bad state (stale CONTEXT.md, missing DEVLOG entries):

1. **Don't panic** — Read ALL available files to reconstruct state
2. **Read the spec** — Ground truth for what the tool should do
3. **Read the code** — Ground truth for what's actually implemented
4. **Reconstruct CONTEXT.md** — Update it based on code analysis
5. **Add a recovery DEVLOG entry** — Document what you found and fixed
6. **Run `make validate-specs`** — Verify integrity
