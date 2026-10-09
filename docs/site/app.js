// ==============================================================================
// tool-scripts Master Documentation Portal — Client Logic & Interactive Engines
// ==============================================================================

document.addEventListener('DOMContentLoaded', () => {
  initReadingProgressBar();
  initScrollSpy();
  initCopyButtons();
  initSearchModal();
  initTokenSimulator();
  initApiPlayground();
  initMermaid();
});

// ─── 1. Reading Progress Bar ──────────────────────────────────────────────────
function initReadingProgressBar() {
  const bar = document.getElementById('reading-progress');
  if (!bar) return;

  window.addEventListener('scroll', () => {
    const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
    if (totalHeight <= 0) return;
    const progress = (window.scrollY / totalHeight) * 100;
    bar.style.width = `${Math.min(100, Math.max(0, progress))}%`;
  }, { passive: true });
}

// ─── 2. ScrollSpy for Sidebar & TOC ───────────────────────────────────────────
function initScrollSpy() {
  const sections = document.querySelectorAll('.chapter-section, .section-h2');
  const navLinks = document.querySelectorAll('.nav-link');
  const tocLinks = document.querySelectorAll('.toc-link');

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        const id = entry.target.id;
        if (!id) return;

        // Update Nav
        navLinks.forEach((link) => {
          if (link.getAttribute('href') === `#${id}`) {
            link.classList.add('active');
          } else {
            link.classList.remove('active');
          }
        });

        // Update TOC
        tocLinks.forEach((link) => {
          if (link.getAttribute('href') === `#${id}`) {
            link.classList.add('active');
          } else {
            link.classList.remove('active');
          }
        });
      }
    });
  }, {
    rootMargin: '-80px 0px -70% 0px',
    threshold: 0
  });

  sections.forEach((sec) => observer.observe(sec));
}

// ─── 3. Copy Code Snippet Buttons ─────────────────────────────────────────────
function initCopyButtons() {
  document.querySelectorAll('.copy-btn').forEach((btn) => {
    btn.addEventListener('click', async () => {
      const codeId = btn.getAttribute('data-target');
      let textToCopy = '';
      if (codeId) {
        const target = document.getElementById(codeId);
        if (target) textToCopy = target.innerText;
      } else {
        const codeBlock = btn.closest('.code-container')?.querySelector('code');
        if (codeBlock) textToCopy = codeBlock.innerText;
      }

      if (textToCopy) {
        try {
          await navigator.clipboard.writeText(textToCopy);
          const orig = btn.innerHTML;
          btn.innerHTML = '<span>✓ Copied!</span>';
          btn.style.color = '#34d399';
          setTimeout(() => {
            btn.innerHTML = orig;
            btn.style.color = '';
          }, 2000);
        } catch (err) {
          console.error('Copy failed', err);
        }
      }
    });
  });
}

// ─── 4. Search Modal & Index ──────────────────────────────────────────────────
const SEARCH_DOCS = [
  { id: 'ch0-overview', title: 'Grand Central: Overview & Monorepo Governance', snippet: 'Monorepo architecture, Spec-Driven Development (SDD), Agentic Constitution.', category: 'Overview' },
  { id: 'ch1-omni-graph', title: 'Omni-Graph: 100% Local Semantic Knowledge Hub', snippet: 'AST graph algebra, 384-dim TEI vector embeddings, HNSW index, SurrealDB v2.', category: 'Omni-Graph' },
  { id: 'omni-architecture', title: 'Omni-Graph: 4-Tier Container Architecture', snippet: 'SurrealDB, HuggingFace TEI, Rust Axum Orchestrator, Cosmograph WebGL Visualizer.', category: 'Omni-Graph' },
  { id: 'omni-ebf', title: 'Ephemeral Branch Fabric (EBF) & Git Worktrees', snippet: 'O(1) worktree peeking, Prior LPA seed inheritance, zero-storm rebase suppression.', category: 'Omni-Graph' },
  { id: 'omni-token-budget', title: 'Two-Tier Token Budgeting Model (<1500 tokens)', snippet: 'Strict character bounding: 3,800 chars for definitions, >=1,800 chars for call traces.', category: 'Omni-Graph' },
  { id: 'omni-polyglot-ast', title: 'Polyglot AST Grammar & Struct Field Extraction', snippet: 'Rust fields/impls/macros, Go methods/structs/fields, TS arrow fns/interfaces/properties.', category: 'Omni-Graph' },
  { id: 'omni-api', title: 'Omni-Graph: Complete REST API Reference', snippet: '/api/symbol, /api/references, /api/condense, /api/query, /api/workspaces, /api/worktrees.', category: 'Omni-Graph' },
  { id: 'ch2-session-explorer', title: 'Session Explorer: Agent Observability & Telemetry', snippet: 'Fast JSONL parser, prompt indexing, tool invocation insights, SQLite cache, Go binary.', category: 'Session Explorer' },
  { id: 'session-architecture', title: 'Session Explorer Architecture & Pipeline', snippet: 'Go backend, Bun frontend, marked.js markdown engine, single standalone binary.', category: 'Session Explorer' },
  { id: 'session-usage', title: 'Session Explorer Usage & CLI Flags', snippet: 'make session-explorer, make run PORT=9876, --no-open headless mode, export markdown.', category: 'Session Explorer' },
  { id: 'ch3-agent-shield', title: 'Agentic Defense Shield & Recon Gates', snippet: 'Pre-invocation hook, pre-tool use gate, 150-line source code reading limit, Recon markers.', category: 'Agent Governance' },
  { id: 'ch4-ci-pipeline', title: 'CI/CD & GitHub Actions Workflows', snippet: 'ci.yml, tool-ci.yml, docs-publish.yml, polyglot test runners, specs validation.', category: 'CI/CD' }
];

function initSearchModal() {
  const modal = document.getElementById('search-modal');
  const trigger = document.getElementById('search-trigger');
  const input = document.getElementById('search-input');
  const resultsContainer = document.getElementById('search-results');

  if (!modal || !trigger || !input || !resultsContainer) return;

  function openModal() {
    modal.classList.add('open');
    input.value = '';
    renderResults('');
    setTimeout(() => input.focus(), 50);
  }

  function closeModal() {
    modal.classList.remove('open');
  }

  trigger.addEventListener('click', openModal);

  document.addEventListener('keydown', (e) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      modal.classList.contains('open') ? closeModal() : openModal();
    } else if (e.key === 'Escape' && modal.classList.contains('open')) {
      closeModal();
    } else if (e.key === '/' && !['INPUT', 'TEXTAREA'].includes(document.activeElement.tagName)) {
      e.preventDefault();
      openModal();
    }
  });

  modal.addEventListener('click', (e) => {
    if (e.target === modal) closeModal();
  });

  input.addEventListener('input', (e) => {
    renderResults(e.target.value.trim().toLowerCase());
  });

  function renderResults(q) {
    resultsContainer.innerHTML = '';
    const filtered = q === '' 
      ? SEARCH_DOCS.slice(0, 6)
      : SEARCH_DOCS.filter(d => 
          d.title.toLowerCase().includes(q) || 
          d.snippet.toLowerCase().includes(q) ||
          d.category.toLowerCase().includes(q)
        );

    if (filtered.length === 0) {
      resultsContainer.innerHTML = '<div style="padding: 24px; text-align: center; color: var(--text-dim);">No matching documentation found</div>';
      return;
    }

    filtered.forEach((item, idx) => {
      const a = document.createElement('a');
      a.className = `search-result-item ${idx === 0 ? 'selected' : ''}`;
      a.href = `#${item.id}`;
      a.innerHTML = `
        <div style="display: flex; justify-content: space-between; align-items: baseline;">
          <div class="result-title">${item.title}</div>
          <span style="font-size: 0.72rem; color: var(--accent-cyan); text-transform: uppercase;">${item.category}</span>
        </div>
        <div class="result-snippet">${item.snippet}</div>
      `;
      a.addEventListener('click', () => closeModal());
      resultsContainer.appendChild(a);
    });
  }
}

// ─── 5. Interactive Token Budget Simulator ────────────────────────────────────
function initTokenSimulator() {
  const rootCharsInput = document.getElementById('sim-root-chars');
  const neighborCharsInput = document.getElementById('sim-neighbor-chars');
  const edgesCountInput = document.getElementById('sim-edges-count');
  const macroCharsInput = document.getElementById('sim-macro-chars');

  if (!rootCharsInput || !neighborCharsInput || !edgesCountInput || !macroCharsInput) return;

  function updateSimulation() {
    const rootChars = parseInt(rootCharsInput.value, 10);
    const neighborChars = parseInt(neighborCharsInput.value, 10);
    const edgesCount = parseInt(edgesCountInput.value, 10);
    const macroChars = parseInt(macroCharsInput.value, 10);

    // Dynamic display labels
    document.getElementById('val-root-chars').innerText = `${rootChars} chars`;
    document.getElementById('val-neighbor-chars').innerText = `${neighborChars} chars`;
    document.getElementById('val-edges-count').innerText = `${edgesCount} relations`;
    document.getElementById('val-macro-chars').innerText = `${macroChars} chars`;

    // Calculations
    const avgEdgeLen = 65; // characters per markdown edge trace: - `src` ──[type]──▶ `tgt`
    const edgesChars = Math.min(edgesCount, 25) * avgEdgeLen;
    const totalChars = rootChars + neighborChars + edgesChars + macroChars;
    const tokenEstimate = Math.ceil(totalChars / 4);
    const maxTokens = 1500;

    document.getElementById('res-total-chars').innerText = totalChars.toLocaleString();
    document.getElementById('res-total-tokens').innerText = `~${tokenEstimate}`;
    
    const tokenDisplay = document.getElementById('res-total-tokens');
    const slaBadge = document.getElementById('res-sla-badge');

    if (tokenEstimate <= maxTokens) {
      tokenDisplay.style.color = '#34d399';
      slaBadge.className = 'sla-badge sla-pass';
      slaBadge.innerText = `SLA PASSED (≤ 1,500 Tokens)`;
    } else {
      tokenDisplay.style.color = '#fb7185';
      slaBadge.className = 'sla-badge sla-fail';
      slaBadge.innerText = `SLA BREACHED (+${tokenEstimate - maxTokens} Tokens)`;
    }
  }

  rootCharsInput.addEventListener('input', updateSimulation);
  neighborCharsInput.addEventListener('input', updateSimulation);
  edgesCountInput.addEventListener('input', updateSimulation);
  macroCharsInput.addEventListener('input', updateSimulation);

  updateSimulation();
}

// ─── 6. Interactive API Playground & Curl Generator ───────────────────────────
function initApiPlayground() {
  const endpointSelect = document.getElementById('api-endpoint');
  const wsInput = document.getElementById('api-ws');
  const symInput = document.getElementById('api-sym');
  const qInput = document.getElementById('api-q');
  const curlBox = document.getElementById('api-curl-output');

  if (!endpointSelect || !wsInput || !symInput || !qInput || !curlBox) return;

  function updateCurl() {
    const ep = endpointSelect.value;
    const ws = wsInput.value.trim() || 'tool-scripts';
    const sym = symInput.value.trim() || 'ContextCondenser';
    const q = qInput.value.trim() || 'How does token budgeting work?';

    let cmd = '';
    switch (ep) {
      case 'query':
        cmd = `curl -s -X POST http://localhost:8080/api/query \\\n  -H "Content-Type: application/json" \\\n  -d '{"prompt": "${q}", "workspace": "${ws}"}'`;
        break;
      case 'condense':
        cmd = `curl -s "http://localhost:8080/api/condense?symbol=${sym}&workspace=${ws}&hops=2"`;
        break;
      case 'symbol':
        cmd = `curl -s "http://localhost:8080/api/symbol?name=${sym}&workspace=${ws}"`;
        break;
      case 'references':
        cmd = `curl -s "http://localhost:8080/api/references?symbol=${sym}&workspace=${ws}"`;
        break;
      case 'galaxies':
        cmd = `curl -s "http://localhost:8080/api/galaxies?workspace=${ws}"`;
        break;
      case 'workspaces':
        cmd = `curl -s "http://localhost:8080/api/workspaces"`;
        break;
      case 'worktrees':
        cmd = `curl -s "http://localhost:8080/api/worktrees"`;
        break;
      case 'ingest':
        cmd = `curl -s -X POST http://localhost:8080/api/ingest \\\n  -H "Content-Type: application/json" \\\n  -d '{"path": "/Users/aparv/workspaces/${ws}", "refresh": true}'`;
        break;
    }

    curlBox.innerText = cmd;
  }

  endpointSelect.addEventListener('change', updateCurl);
  wsInput.addEventListener('input', updateCurl);
  symInput.addEventListener('input', updateCurl);
  qInput.addEventListener('input', updateCurl);

  updateCurl();
}

// ─── 7. Mermaid Diagrams Rendering ────────────────────────────────────────────
function initMermaid() {
  if (window.mermaid) {
    mermaid.initialize({
      startOnLoad: true,
      theme: 'dark',
      themeVariables: {
        darkMode: true,
        background: '#090d16',
        primaryColor: '#00f2fe',
        primaryTextColor: '#f8fafc',
        primaryBorderColor: '#38bdf8',
        lineColor: '#a855f7',
        secondaryColor: '#1e293b',
        tertiaryColor: '#0f172a'
      }
    });
  }
}
