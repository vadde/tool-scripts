---
name: docs-portal
description: >-
  Authoritative engineering guide and automated testing protocol for the tool-scripts Living Documentation Portal. Covers liquid-glass design system, 1:1 bi-directional navigation sync, Mermaid diagram canvas invariants, KaTeX mathematical typesetting, interactive simulators, and headless Chrome verification.
---

# 📚 Living Documentation Portal Engineering Skill

This skill is the **single source of truth** for maintaining, styling, extending, and testing the `vadde/tool-scripts` Living Documentation Portal ([https://vadde.github.io/tool-scripts/](https://vadde.github.io/tool-scripts/)).

Follow this guide whenever adding chapters, modifying CSS styles, rendering architecture diagrams, typesetting mathematical equations, or building interactive simulator widgets.

---

## 🏛️ Portal Architecture & File Map

The documentation portal is built with **zero build toolchain dependencies** (pure vanilla HTML5, CSS3, ES2022 JavaScript, and CDN-loaded libraries):

```
tool-scripts/
├── docs/site/
│   ├── index.html        → Semantic HTML5 layout, chapter disclosure trees, widgets
│   ├── style.css         → Liquid-glass design tokens, animations, canvas layout
│   └── app.js            → 1:1 ScrollSpy, zoomable canvas pan/zoom, simulator engines
├── scripts/
│   └── build-docs.sh     → Static bundle packaging script (outputs to _site/)
├── .agents/skills/docs-portal/
│   ├── SKILL.md          → YOU ARE HERE — Authoritative design & engineering protocol
│   └── scripts/
│       └── test-docs.sh  → Automated headless Chrome verification suite
└── .github/workflows/
    └── docs-publish.yml  → CI/CD workflow deploying main pushes to GitHub Pages
```

---

## 🎨 Design System & Color Tokens

The portal uses a **deep cosmic liquid-glass design system**. Always use CSS variables rather than hardcoding ad-hoc color values:

### Color Palette

| Token | CSS Variable | Hex / RGBA Value | Purpose |
|---|---|---|---|
| **Canvas Deep Base** | `--bg-deep` | `#070a13` | Deep space background canvas |
| **Card Surface** | `--bg-card` | `rgba(15, 23, 42, 0.75)` | Glassmorphism card background |
| **Card Border** | `--bg-card-border` | `rgba(255, 255, 255, 0.08)` | Subtle translucent glass card edge |
| **Hover Border** | `--bg-card-border-hover` | `rgba(0, 242, 254, 0.4)` | Interactive element border focus |
| **Cyan Accent** | `--accent-cyan` | `#00f2fe` | Primary interactive focus, metrics, active items |
| **Violet Accent** | `--accent-violet` | `#a855f7` | Diagram relational links, secondary badges |
| **Emerald Accent** | `--accent-emerald` | `#10b981` | Passing SLA badges, Rust indicators, health |
| **Amber Accent** | `--accent-amber` | `#f59e0b` | Cautionary callouts, architectural invariant warnings |
| **Rose Accent** | `--accent-rose` | `#f43f5e` | SLA breaches, error indicators, fatal alerts |
| **Primary Text** | `--text-primary` | `#f8fafc` | High-contrast body text and titles |
| **Muted Text** | `--text-muted` | `#94a3b8` | Subtitles, parameter descriptions |
| **Dim Text** | `--text-dim` | `#64748b` | Captions, footnotes, table metadata |

### Glassmorphism Recipe
```css
/* Canonical frosted glass surface */
background: rgba(15, 23, 42, 0.75);
backdrop-filter: blur(20px);
-webkit-backdrop-filter: blur(20px);
border: 1px solid var(--bg-card-border);
border-radius: var(--radius-lg);
```

### Typography Hierarchy
- **Display Headings (`h1`, `h2`, `.chapter-title`)**: `'Outfit', sans-serif`, font-weight 700–800.
- **Body & Controls**: `'Plus Jakarta Sans', sans-serif`, font-weight 400–600.
- **Code, Metrics & Data**: `'JetBrains Mono', monospace`, font-weight 500–700.

---

## 🧭 Navigation & The 1:1 Synchrony Invariant

### 1. The Structure of Navigation
The left sidebar is composed of `.nav-tree-group` sections containing:
1. `.nav-tree-header`: Chapter group bar with disclosure chevron and language pill.
2. `.nav-tree-children`: List of `.nav-tree-link` anchors (`href="#sec-id"`).

### 2. The Bi-Directional Synchrony Contract
* **Header Click**: Clicking `.nav-tree-header` toggles `.expanded` on the sidebar group **AND** simultaneously toggles `.chapter-expanded` on the corresponding `<section class="chapter-section">` in the content pane.
* **Child Link Click**: Clicking a `.nav-tree-link`:
  1. Ensures containing parent chapter is expanded.
  2. Applies Focus Mode rules (collapses sibling chapters if Focus Mode is active).
  3. Smooth-scrolls to the target element with sticky-header offset (`160px`).
  4. Applies temporary `.flash-focus` neon glow (1.8s) to the target element.
  5. Sets `isNavClicking = true` for `850ms` to prevent smooth-scroll events from overriding the active highlight.
* **ScrollSpy Tracking (CRITICAL)**:
  - ScrollSpy tracks **EVERY TARGET ELEMENT LINKED IN THE NAVIGATION** individually, NOT just `.chapter-section`!
  - Targets include: `#sec-omni-arch`, `#sec-omni-ebf`, `#sec-omni-ast`, `#sec-omni-budget`, `#sec-omni-relational`, etc.
  - **INVARIANT**: ScrollSpy must **NEVER** mutate tree open/close state (`classList.add('expanded')`) while scrolling! It ONLY updates `.active` on the link and `.active-branch` on the group header.
  - **INVARIANT**: When "Collapse All" is clicked, scrolling to the bottom must **NEVER** forcefully pop open Sections 00 or 01.

---

## 📊 Mermaid Architecture Diagrams & Zoomable Canvas

### 1. Canvas DOM Anatomy
Every architecture diagram and complex table must be wrapped in a zoomable canvas:
```html
<div class="canvas-container">
  <div class="canvas-header-bar">
    <span class="canvas-type-tag">Architecture Blueprint · 4-Tier Container Stack</span>
    <div class="canvas-controls-group">
      <button class="canvas-tool-btn zoom-out-btn" title="Zoom Out">−</button>
      <span class="zoom-level-badge">100%</span>
      <button class="canvas-tool-btn zoom-in-btn" title="Zoom In">+</button>
      <button class="canvas-tool-btn zoom-reset-btn" title="Reset Zoom">⟲</button>
      <button class="canvas-tool-btn fullscreen-btn" title="View Fullscreen Theater">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"></path></svg>
        <span>Theater</span>
      </button>
    </div>
  </div>
  <div class="canvas-viewport">
    <div class="canvas-content-surface">
      <pre class="mermaid">
flowchart TB
    ...
      </pre>
    </div>
  </div>
  <div class="canvas-caption">Figure 1.1: Caption text here</div>
</div>
```

### 2. The Critical SVG Layout Invariant (DO NOT VIOLATE)
> [!CAUTION]
> **NEVER apply `width: auto !important` or `height: auto !important` to `.mermaid svg` inside nested flex containers!**
> In modern browser rendering engines (WebKit, Blink), overriding SVG width/height with `auto` inside flex containers collapses the SVG's flex basis to **0px by 0px**, rendering diagrams completely invisible.

**The Golden Layout Rules for Mermaid:**
```css
/* 1. Viewport handles overflow and scroll */
.canvas-viewport {
  overflow: auto;
  padding: 28px 24px;
  display: block;
  text-align: center;
  min-height: 220px;
  max-height: 600px;
  cursor: grab;
  user-select: none;
}

/* 2. Surfaces flow as inline-block */
.canvas-content-surface {
  transform-origin: center top;
  transition: transform 0.12s ease-out;
  display: inline-block;
  margin: 0 auto;
  text-align: center;
  max-width: 100%;
}

.mermaid {
  display: inline-block !important;
  margin: 0 auto !important;
  text-align: center !important;
  overflow: visible !important;
}

/* 3. SVG respects its native responsive coordinate space */
.mermaid svg {
  display: inline-block !important;
  max-width: 100% !important;
  height: auto !important;
}
```

### 3. Mermaid Node Label Escaping Rule
> [!WARNING]
> Unescaped `<` or `>` characters inside Mermaid node labels (e.g. `[Condenser (<1500 Tokens)]`) are parsed as HTML tags and **crash the Mermaid parser**!
> **Always write:** `≤ 1,500 Tokens` or `&lt;1500 Tokens`.

### 4. Theater Button Layout Rule
The theater button contains an SVG icon and text ("Theater"). Its CSS class `.canvas-tool-btn.fullscreen-btn` must specify:
`min-width: 32px; padding: 0 14px; gap: 8px; font-weight: 600;`
Never constrain it to `width: 28px` (which is reserved for single-character zoom buttons like `+` and `−`).

---

## 📐 Mathematical Typesetting (KaTeX)

All LaTeX mathematical expressions must render crisply in dark glass containers:
- **Inline Math**: Wrapped in `$ ... $` (e.g. `$\le 1,500$`, `$\approx 1$ms`, `$C_{\text{max}}$`).
- **Display Math**: Wrapped in `$$ ... $$` (e.g. `$$C_{\text{total}} = C_{\text{root}} + \dots$$`).

### KaTeX Invariants
1. Loaded in `<head>` via jsDelivr CDN (`katex.min.css`, `katex.min.js`, `auto-render.min.js`).
2. Auto-render extension must ignore preformatted code blocks:
   ```javascript
   ignoredTags: ["script", "noscript", "style", "textarea", "pre", "code", "annotation"]
   ```
   *Why this matters:* If `<pre>` is not ignored, KaTeX will attempt to parse Mermaid diagram source code and destroy the diagrams before Mermaid executes!

---

## 🎛️ Interactive Simulators & Playgrounds

Interactive modules (such as the **Context Condenser Token Simulator** and **API Playground**) are first-class citizens of Chapter 01:

### 1. Module Placement & Disclosure
- Must use `<section class="chapter-section" id="token-simulator">` with standard `.chapter-header-trigger`.
- Must include a badge: `Chapter 01 · Interactive Module 1.1`.
- Must expand and collapse in harmony with "Expand All", "Collapse All", and "Focus Mode".

### 2. Parameter Control Card Anatomy (The 2-Line Rule)
Never squeeze the parameter title and value into a single row with `justify-content: space-between`. On narrow cards or multi-word titles, this forces ugly multi-line wrapping (e.g. `Neighbor \n Cap` and `4500 \n chars`).

**Always use the 2-Line Stack inside isolated frosted cards:**
```html
<div class="control-item">
  <!-- Line 1: Parameter Title -->
  <div class="control-title">Neighbor Signatures Cap</div>
  <!-- Line 2: Prominent Monospace Quantitative Value -->
  <div class="control-value" id="val-neighbor-chars">2400 chars</div>
  <!-- Line 3: Range Slider Track -->
  <input type="range" id="sim-neighbor-chars" min="500" max="4500" step="100" value="2400">
</div>
```

```css
.control-item {
  display: flex;
  flex-direction: column;
  background: rgba(15, 23, 42, 0.65);
  border: 1px solid var(--bg-card-border);
  border-radius: var(--radius-md);
  padding: 16px 18px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  transition: border-color 0.2s, box-shadow 0.2s;
}

.control-item .control-title {
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 6px;
}

.control-item .control-value {
  font-family: var(--font-mono);
  font-size: 1.18rem;
  font-weight: 700;
  color: var(--accent-cyan);
  margin-bottom: 12px;
}

.control-item input[type="range"] {
  width: 100%;
  accent-color: var(--accent-cyan);
  cursor: pointer;
  margin-top: auto;
}
```

---

## 🧪 Automated Testing Protocol (Zero Guesswork)

Never push documentation portal changes without running the automated test suite!

### 1. Run the Automated Verification Suite
```bash
# Run the all-in-one headless audit:
bash .agents/skills/docs-portal/scripts/test-docs.sh

# Or via the root Makefile target:
make docs-test
```

The script automatically executes:
1. **Core Asset Presence**: Verifies `index.html`, `style.css`, and `app.js`.
2. **Mermaid Syntax Auditing**: Detects unescaped `<` or `>` characters in node labels.
3. **SVG Sizing Invariants**: Validates that CSS does not contain collapsing `width: auto !important` on `.mermaid svg`.
4. **KaTeX Integration**: Verifies stylesheet, scripts, and auto-render calls.
5. **Static Bundle Build**: Compiles to `_site/` and checks size/file counts.
6. **Headless Chrome Render Test**: Spins up an ephemeral local server, launches headless Chrome, dumps the rendered DOM, and verifies that **all 3 Mermaid SVGs** rendered with non-zero dimensions.

### 2. Manual Local Preview
```bash
# Build and serve locally on http://localhost:4000:
make docs-serve

# Or direct Python server:
python3 -m http.server 4000 --directory _site
```

### 3. Verify Live Deployment
After pushing to `main`:
```bash
# 1. Watch GitHub Actions workflow:
gh run list --workflow="docs-publish.yml" --limit 1

# 2. Verify HTTP 200 on live GitHub Pages:
curl -sI https://vadde.github.io/tool-scripts/
```

---

## 🚀 Playbook: Adding a New Chapter or Tool

When documenting a new tool (e.g. `Chapter 04 · New Tool`):

1. **Update Left Navigation (`docs/site/index.html`)**:
   Add a new `.nav-tree-group`:
   ```html
   <div class="nav-tree-group">
     <div class="nav-tree-header" data-target="#ch4-new-tool">
       <div class="tree-title-group">
         <span class="tree-chevron">▶</span>
         <span class="tree-icon">🚀</span>
         <span class="tree-title-text">04. New Tool Name</span>
       </div>
       <span class="nav-pill pill-rust">Rust</span>
     </div>
     <div class="nav-tree-children">
       <a href="#ch4-new-tool" class="nav-tree-link">System Overview</a>
       <a href="#sec-new-arch" class="nav-tree-link">Architecture Topology</a>
       <a href="#sec-new-usage" class="nav-tree-link">Usage & Quickstart</a>
     </div>
   </div>
   ```

2. **Add Chapter Section (`docs/site/index.html`)**:
   ```html
   <section class="chapter-section" id="ch4-new-tool">
     <div class="chapter-header-trigger">
       <div>
         <div class="chapter-badge">Chapter 04 · Tool Category</div>
         <h2 class="chapter-title">Tool Name: Purpose</h2>
         <div style="font-size: 0.85rem; color: var(--text-dim);">Subtitle</div>
       </div>
       <div class="chapter-toggle-indicator">
         <div class="chapter-toggle-icon">▼</div>
       </div>
     </div>
     <div class="chapter-body">
       <p>Overview text...</p>
       <h3 class="section-h2" id="sec-new-arch">Architecture Topology</h3>
       <!-- Diagram Canvas here -->
       <h3 class="section-h2" id="sec-new-usage">Usage & Quickstart</h3>
       <!-- Code block here -->
     </div>
   </section>
   ```

3. **Verify and Deploy**:
   ```bash
   bash .agents/skills/docs-portal/scripts/test-docs.sh
   git commit -am "feat(docs): add Chapter 04 documentation"
   git push origin main
   ```
