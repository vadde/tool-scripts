// ==============================================================================
// tool-scripts Master Documentation Portal — Client Logic & Interactive Engines
// ==============================================================================

let isFocusModeActive = false; // When true, expanding one chapter auto-collapses others

document.addEventListener('DOMContentLoaded', () => {
  initReadingProgressBar();
  initTreeNavigation();
  initCollapsibleChapters();
  initCanvasZoomAndTheater();
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

// ─── 2. Hierarchical Tree Navigation & Sidebar Rail ───────────────────────────
function initTreeNavigation() {
  const sidebar = document.getElementById('main-sidebar');
  const railToggle = document.getElementById('sidebar-rail-toggle');
  const expandAllBtn = document.getElementById('expand-all-trees');
  const collapseAllBtn = document.getElementById('collapse-all-trees');

  // Toggle Sidebar Compact Rail
  if (railToggle && sidebar) {
    railToggle.addEventListener('click', () => {
      sidebar.classList.toggle('rail-collapsed');
      const isCollapsed = sidebar.classList.contains('rail-collapsed');
      railToggle.setAttribute('title', isCollapsed ? 'Expand Navigation Sidebar' : 'Collapse Navigation to Rail');
    });
  }

  // Expand / Collapse Tree Group Headers
  document.querySelectorAll('.nav-tree-header').forEach((header) => {
    header.addEventListener('click', (e) => {
      const group = header.closest('.nav-tree-group');
      if (group) {
        group.classList.toggle('expanded');
      }
    });
  });

  // Expand All / Collapse All Tree Buttons
  if (expandAllBtn) {
    expandAllBtn.addEventListener('click', () => {
      document.querySelectorAll('.nav-tree-group').forEach(g => g.classList.add('expanded'));
    });
  }

  if (collapseAllBtn) {
    collapseAllBtn.addEventListener('click', () => {
      document.querySelectorAll('.nav-tree-group').forEach(g => g.classList.remove('expanded'));
    });
  }

  // When clicking any link in tree navigation, auto-expand its target chapter section
  document.querySelectorAll('.nav-tree-link').forEach((link) => {
    link.addEventListener('click', (e) => {
      const href = link.getAttribute('href');
      if (href && href.startsWith('#')) {
        const targetSec = document.querySelector(href);
        if (targetSec) {
          ensureSectionExpanded(targetSec);
        }
      }
    });
  });
}

// ─── 3. Collapsible Chapters & Accordion / Focus Modes ────────────────────────
function initCollapsibleChapters() {
  const expandAllBtn = document.getElementById('mode-expand-all');
  const focusModeBtn = document.getElementById('mode-focus-accordion');
  const collapseAllBtn = document.getElementById('mode-collapse-all');

  // Chapter Header Accordion Click Handlers
  document.querySelectorAll('.chapter-header-trigger').forEach((trigger) => {
    trigger.addEventListener('click', () => {
      const section = trigger.closest('.chapter-section');
      if (!section) return;

      const willExpand = section.classList.contains('is-collapsed');

      if (willExpand) {
        if (isFocusModeActive) {
          // In Focus Mode, collapse all other chapters
          document.querySelectorAll('.chapter-section').forEach(s => {
            if (s !== section) s.classList.add('is-collapsed');
          });
        }
        section.classList.remove('is-collapsed');
      } else {
        section.classList.add('is-collapsed');
      }
    });
  });

  // Reading Mode Buttons
  if (expandAllBtn) {
    expandAllBtn.addEventListener('click', () => {
      isFocusModeActive = false;
      setActiveModeBtn(expandAllBtn);
      document.querySelectorAll('.chapter-section').forEach(s => s.classList.remove('is-collapsed'));
    });
  }

  if (focusModeBtn) {
    focusModeBtn.addEventListener('click', () => {
      isFocusModeActive = true;
      setActiveModeBtn(focusModeBtn);
      // Collapse all except the first or currently visible
      const sections = Array.from(document.querySelectorAll('.chapter-section'));
      if (sections.length > 0) {
        sections[0].classList.remove('is-collapsed');
        for (let i = 1; i < sections.length; i++) {
          sections[i].classList.add('is-collapsed');
        }
      }
    });
  }

  if (collapseAllBtn) {
    collapseAllBtn.addEventListener('click', () => {
      isFocusModeActive = false;
      setActiveModeBtn(collapseAllBtn);
      document.querySelectorAll('.chapter-section').forEach(s => s.classList.add('is-collapsed'));
    });
  }

  function setActiveModeBtn(activeBtn) {
    [expandAllBtn, focusModeBtn, collapseAllBtn].forEach(btn => {
      if (btn) btn.classList.remove('active');
    });
    if (activeBtn) activeBtn.classList.add('active');
  }
}

// Helper: Ensure a target section is expanded when navigated to
function ensureSectionExpanded(targetElement) {
  const parentSection = targetElement.closest('.chapter-section') || targetElement;
  if (parentSection && parentSection.classList.contains('is-collapsed')) {
    if (isFocusModeActive) {
      document.querySelectorAll('.chapter-section').forEach(s => {
        if (s !== parentSection) s.classList.add('is-collapsed');
      });
    }
    parentSection.classList.remove('is-collapsed');
  }
}

// ─── 4. Interactive Zoomable Visual Canvas & Fullscreen Theater ───────────────
function initCanvasZoomAndTheater() {
  const theaterModal = document.getElementById('theater-modal');
  const theaterTitle = document.getElementById('theater-title');
  const theaterSurface = document.getElementById('theater-surface');
  const theaterViewport = document.getElementById('theater-viewport');
  const closeTheaterBtn = document.getElementById('close-theater-btn');
  const theaterZoomIn = document.getElementById('theater-zoom-in');
  const theaterZoomOut = document.getElementById('theater-zoom-out');
  const theaterZoomReset = document.getElementById('theater-zoom-reset');
  const theaterZoomBadge = document.getElementById('theater-zoom-badge');

  let activeTheaterZoom = 1.0;
  let activeTheaterPanX = 0;
  let activeTheaterPanY = 0;

  // Initialize all canvas containers
  document.querySelectorAll('.canvas-container').forEach((canvas) => {
    const viewport = canvas.querySelector('.canvas-viewport');
    const surface = canvas.querySelector('.canvas-content-surface');
    const zoomInBtn = canvas.querySelector('.zoom-in-btn');
    const zoomOutBtn = canvas.querySelector('.zoom-out-btn');
    const zoomResetBtn = canvas.querySelector('.zoom-reset-btn');
    const zoomBadge = canvas.querySelector('.zoom-level-badge');
    const fullscreenBtn = canvas.querySelector('.fullscreen-btn');
    const titleTag = canvas.querySelector('.canvas-type-tag')?.innerText || 'Diagram View';

    if (!viewport || !surface) return;

    let zoom = 1.0;
    let panX = 0;
    let panY = 0;
    let isPanning = false;
    let startX = 0;
    let startY = 0;

    function applyTransform() {
      surface.style.transform = `scale(${zoom}) translate(${panX}px, ${panY}px)`;
      if (zoomBadge) zoomBadge.innerText = `${Math.round(zoom * 100)}%`;
    }

    if (zoomInBtn) {
      zoomInBtn.addEventListener('click', () => {
        zoom = Math.min(3.5, zoom + 0.25);
        applyTransform();
      });
    }

    if (zoomOutBtn) {
      zoomOutBtn.addEventListener('click', () => {
        zoom = Math.max(0.5, zoom - 0.25);
        applyTransform();
      });
    }

    if (zoomResetBtn) {
      zoomResetBtn.addEventListener('click', () => {
        zoom = 1.0;
        panX = 0;
        panY = 0;
        applyTransform();
      });
    }

    // Draggable panning
    viewport.addEventListener('mousedown', (e) => {
      if (e.target.closest('button')) return;
      isPanning = true;
      startX = e.clientX - panX;
      startY = e.clientY - panY;
      viewport.style.cursor = 'grabbing';
    });

    window.addEventListener('mousemove', (e) => {
      if (!isPanning) return;
      panX = e.clientX - startX;
      panY = e.clientY - startY;
      applyTransform();
    });

    window.addEventListener('mouseup', () => {
      if (isPanning) {
        isPanning = false;
        viewport.style.cursor = 'grab';
      }
    });

    // Mouse wheel zoom with Ctrl or Meta
    viewport.addEventListener('wheel', (e) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.15 : -0.15;
        zoom = Math.min(3.5, Math.max(0.5, zoom + delta));
        applyTransform();
      }
    }, { passive: false });

    // Open Fullscreen Theater
    if (fullscreenBtn && theaterModal && theaterSurface) {
      fullscreenBtn.addEventListener('click', () => {
        openTheater(titleTag, surface.innerHTML);
      });
    }
  });

  // Theater Logic
  function openTheater(title, innerContent) {
    if (!theaterModal || !theaterSurface) return;
    theaterTitle.innerText = title;
    theaterSurface.innerHTML = innerContent;
    activeTheaterZoom = 1.25; // Default larger scale in theater
    activeTheaterPanX = 0;
    activeTheaterPanY = 0;
    updateTheaterTransform();

    theaterModal.classList.add('theater-open');
    document.body.style.overflow = 'hidden';
  }

  function closeTheater() {
    if (!theaterModal) return;
    theaterModal.classList.remove('theater-open');
    document.body.style.overflow = '';
  }

  function updateTheaterTransform() {
    if (!theaterSurface) return;
    theaterSurface.style.transform = `scale(${activeTheaterZoom}) translate(${activeTheaterPanX}px, ${activeTheaterPanY}px)`;
    if (theaterZoomBadge) theaterZoomBadge.innerText = `${Math.round(activeTheaterZoom * 100)}%`;
  }

  if (closeTheaterBtn) closeTheaterBtn.addEventListener('click', closeTheater);

  if (theaterZoomIn) {
    theaterZoomIn.addEventListener('click', () => {
      activeTheaterZoom = Math.min(4.0, activeTheaterZoom + 0.25);
      updateTheaterTransform();
    });
  }

  if (theaterZoomOut) {
    theaterZoomOut.addEventListener('click', () => {
      activeTheaterZoom = Math.max(0.5, activeTheaterZoom - 0.25);
      updateTheaterTransform();
    });
  }

  if (theaterZoomReset) {
    theaterZoomReset.addEventListener('click', () => {
      activeTheaterZoom = 1.0;
      activeTheaterPanX = 0;
      activeTheaterPanY = 0;
      updateTheaterTransform();
    });
  }

  // Panning inside Theater
  let isTheaterPanning = false;
  let theaterStartX = 0;
  let theaterStartY = 0;

  if (theaterViewport) {
    theaterViewport.addEventListener('mousedown', (e) => {
      if (e.target.closest('button')) return;
      isTheaterPanning = true;
      theaterStartX = e.clientX - activeTheaterPanX;
      theaterStartY = e.clientY - activeTheaterPanY;
      theaterViewport.style.cursor = 'grabbing';
    });

    window.addEventListener('mousemove', (e) => {
      if (!isTheaterPanning) return;
      activeTheaterPanX = e.clientX - theaterStartX;
      activeTheaterPanY = e.clientY - theaterStartY;
      updateTheaterTransform();
    });

    window.addEventListener('mouseup', () => {
      if (isTheaterPanning) {
        isTheaterPanning = false;
        if (theaterViewport) theaterViewport.style.cursor = 'grab';
      }
    });

    theaterViewport.addEventListener('wheel', (e) => {
      if (e.ctrlKey || e.metaKey) {
        e.preventDefault();
        const delta = e.deltaY < 0 ? 0.15 : -0.15;
        activeTheaterZoom = Math.min(4.0, Math.max(0.5, activeTheaterZoom + delta));
        updateTheaterTransform();
      }
    }, { passive: false });
  }

  // Close theater on Escape key
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && theaterModal?.classList.contains('theater-open')) {
      closeTheater();
    }
  });
}

// ─── 5. ScrollSpy for Sidebar & TOC ───────────────────────────────────────────
function initScrollSpy() {
  const sections = document.querySelectorAll('.chapter-section, .section-h2');
  const navTreeLinks = document.querySelectorAll('.nav-tree-link');
  const tocLinks = document.querySelectorAll('.toc-link');

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        const id = entry.target.id;
        if (!id) return;

        // Update Tree Nav Link & Active Branch Group
        navTreeLinks.forEach((link) => {
          if (link.getAttribute('href') === `#${id}`) {
            link.classList.add('active');
            const parentGroup = link.closest('.nav-tree-group');
            if (parentGroup) {
              parentGroup.classList.add('expanded');
              parentGroup.querySelector('.nav-tree-header')?.classList.add('active-branch');
            }
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

// ─── 6. Copy Code Snippet Buttons ─────────────────────────────────────────────
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

// ─── 7. Search Modal & Index ──────────────────────────────────────────────────
const SEARCH_DOCS = [
  { id: 'ch0-overview', title: 'Grand Central: Overview & Monorepo Governance', snippet: 'Monorepo architecture, Spec-Driven Development (SDD), Agentic Constitution.', category: 'Overview' },
  { id: 'sec-sdd-governance', title: 'Spec-Driven Development (SDD) & SDLC Finite State Machine', snippet: 'Contract before code, numbered requirements (R-XXX), status verification gates.', category: 'Governance' },
  { id: 'sec-agent-shield', title: 'The Agentic Defense Shield & Reconnaissance Gates', snippet: '3-tier recon ladder, pre-invocation hook, 150-line gate, recon markers.', category: 'Agent Shield' },
  { id: 'ch1-omni-graph', title: 'Omni-Graph: 100% Local Semantic Knowledge Hub', snippet: 'AST graph algebra, 384-dim TEI vector embeddings, HNSW index, SurrealDB v2.', category: 'Omni-Graph' },
  { id: 'sec-omni-arch', title: 'Omni-Graph: 4-Tier Container Architecture', snippet: 'SurrealDB, HuggingFace TEI, Rust Axum Orchestrator, Cosmograph WebGL Visualizer.', category: 'Omni-Graph' },
  { id: 'sec-omni-ebf', title: 'Ephemeral Branch Fabric (EBF) & Git Worktrees', snippet: 'O(1) worktree peeking, Prior LPA seed inheritance, zero-storm rebase suppression.', category: 'Omni-Graph' },
  { id: 'sec-omni-ast', title: 'Polyglot AST Grammar & Struct Field Extraction', snippet: 'Rust fields/impls/macros, Go methods/structs/fields, TS arrow fns/interfaces/properties.', category: 'Omni-Graph' },
  { id: 'sec-omni-budget', title: 'Two-Tier Context Condenser Token Budget Math (<1500 tokens)', snippet: 'Strict character bounding: 3,800 chars for definitions, >=1,800 chars for call traces.', category: 'Omni-Graph' },
  { id: 'sec-omni-relational', title: 'SurrealDB Relational Indexing & Zero-Blackout Re-indexing', snippet: 'idx_edge_in/out indexes, atomic vector generation before node pruning.', category: 'Omni-Graph' },
  { id: 'token-simulator', title: 'Interactive Context Condenser Token Simulator', snippet: 'Simulate real-time token counts, root snippets, neighbor caps, and SLA compliance.', category: 'Interactive' },
  { id: 'api-playground', title: 'Omni-Graph Live API Playground & Curl Generator', snippet: '/api/symbol, /api/references, /api/condense, /api/query, /api/workspaces, /api/worktrees.', category: 'Interactive' },
  { id: 'ch2-session-explorer', title: 'Session Explorer: Agent Observability & Telemetry', snippet: 'Fast JSONL parser, prompt indexing, tool invocation insights, SQLite cache, Go binary.', category: 'Session Explorer' },
  { id: 'sec-session-arch', title: 'Session Explorer: Go + Bun Standalone Binary Architecture', snippet: 'Go backend, Bun frontend, marked.js markdown engine, single standalone binary.', category: 'Session Explorer' },
  { id: 'sec-session-usage', title: 'Session Explorer: Telemetry & CLI Quickstart', snippet: 'make session-explorer, make run PORT=9876, --no-open headless mode, export markdown.', category: 'Session Explorer' },
  { id: 'ch3-ci-pipeline', title: 'CI/CD & GitHub Actions Workflows', snippet: 'ci.yml, tool-ci.yml, docs-publish.yml, polyglot test runners, specs validation.', category: 'CI/CD' },
  { id: 'sec-docs-drift', title: 'The Documentation Synchronization Gate', snippet: '4-point documentation gate, zero drift policy, Rule 06 standards.', category: 'Governance' }
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
      a.addEventListener('click', () => {
        closeModal();
        const targetSec = document.getElementById(item.id);
        if (targetSec) ensureSectionExpanded(targetSec);
      });
      resultsContainer.appendChild(a);
    });
  }
}

// ─── 8. Interactive Token Budget Simulator ────────────────────────────────────
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

// ─── 9. Interactive API Playground & Curl Generator ───────────────────────────
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

// ─── 10. Mermaid Diagrams Rendering ───────────────────────────────────────────
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
