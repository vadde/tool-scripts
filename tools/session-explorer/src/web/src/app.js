// ==========================================================================
// Session Explorer — Single Page Application Logic
// Implements: R-008..R-016, R-021, R-022, R-025, R-026
// ==========================================================================

import { marked } from 'marked';
import hljs from 'highlight.js';

// Configure marked with highlight.js syntax highlighter
const renderer = {
  code({ text, lang }) {
    const validLang = !!(lang && hljs.getLanguage(lang));
    const highlighted = validLang
      ? hljs.highlight(text, { language: lang }).value
      : hljs.highlightAuto(text).value;
    return `<pre><code class="hljs ${validLang ? 'language-' + lang : ''}">${highlighted}</code></pre>`;
  }
};

marked.use({ renderer, breaks: true, gfm: true });

// App State
const state = {
  sessions: [],
  stats: null,
  selectedSessionId: null,
  selectedSessionDetail: null,
  currentView: 'dashboard', // 'dashboard' | 'detail'
  filterWorkspace: '',
  dateFrom: '',
  dateTo: '',
  sortBy: 'date',
  sortOrder: 'desc',
  messageFilter: 'ALL', // 'ALL' | 'USER_INPUT' | 'PLANNER_RESPONSE' | 'TOOL_CALLS' | 'ERROR'
  searchQuery: '',
  searchResults: [],
  theme: localStorage.getItem('se_theme') || 'dark',
  drilledProject: null, // active project when drilled into a repository
  projectFilters: {
    search: '',
    sort: 'date_desc',
    dateFrom: '',
    dateTo: '',
    datePreset: 'all'
  }
};

// DOM Elements
const el = {
  themeToggleBtn: document.getElementById('themeToggleBtn'),
  insightsBtn: document.getElementById('insightsBtn'),
  refreshBtn: document.getElementById('refreshBtn'),
  globalSearchInput: document.getElementById('globalSearchInput'),
  statsStrip: document.getElementById('statsStrip'),
  workspaceSelect: document.getElementById('workspaceSelect'),
  workspaceStrip: document.getElementById('workspaceStrip'),
  sortBySelect: document.getElementById('sortBySelect'),
  dateFromInput: document.getElementById('dateFromInput'),
  dateToInput: document.getElementById('dateToInput'),
  sessionCountBadge: document.getElementById('sessionCountBadge'),
  sessionGrid: document.getElementById('sessionGrid'),
  viewContainer: document.getElementById('viewContainer'),
  groupByControl: document.getElementById('groupByControl'),
  groupByBtns: document.querySelectorAll('#groupByControl .segment-btn'),
  projectHeroBanner: document.getElementById('projectHeroBanner'),
  dashboardView: document.getElementById('dashboardView'),
  detailView: document.getElementById('detailView'),
  detailBackBtn: document.getElementById('detailBackBtn'),
  detailSessionId: document.getElementById('detailSessionId'),
  detailWorkspaceBadge: document.getElementById('detailWorkspaceBadge'),
  detailTime: document.getElementById('detailTime'),
  exportMdBtn: document.getElementById('exportMdBtn'),
  exportJsonBtn: document.getElementById('exportJsonBtn'),
  timelineContainer: document.getElementById('timelineContainer'),
  msgFilterChips: document.querySelectorAll('.msg-filter-chip'),
  datePresets: document.querySelectorAll('.date-preset-chip'),
  searchModal: document.getElementById('searchModal'),
  modalSearchInput: document.getElementById('modalSearchInput'),
  modalSearchResults: document.getElementById('modalSearchResults'),
  closeSearchModalBtn: document.getElementById('closeSearchModalBtn'),
  insightsModal: document.getElementById('insightsModal'),
  closeInsightsModalBtn: document.getElementById('closeInsightsModalBtn'),
  topToolsList: document.getElementById('topToolsList'),
  topPromptsList: document.getElementById('topPromptsList'),
  toast: document.getElementById('toastNotification')
};

// Initialize App
document.addEventListener('DOMContentLoaded', () => {
  initTheme();
  initEventListeners();
  loadSessions();
});

// Theme Management
function initTheme() {
  document.documentElement.setAttribute('data-theme', state.theme);
  updateThemeIcon();
}

function toggleTheme() {
  state.theme = state.theme === 'dark' ? 'light' : 'dark';
  document.documentElement.setAttribute('data-theme', state.theme);
  localStorage.setItem('se_theme', state.theme);
  updateThemeIcon();
}

function updateThemeIcon() {
  if (el.themeToggleBtn) {
    el.themeToggleBtn.textContent = state.theme === 'dark' ? '☀️' : '🌙';
  }
}

// Toast Notifications (R-026)
function showToast(message) {
  if (!el.toast) return;
  el.toast.textContent = message;
  el.toast.classList.add('active');
  setTimeout(() => {
    el.toast.classList.remove('active');
  }, 2500);
}

// Event Listeners
function initEventListeners() {
  if (el.themeToggleBtn) el.themeToggleBtn.addEventListener('click', toggleTheme);
  if (el.refreshBtn) el.refreshBtn.addEventListener('click', () => loadSessions());
  if (el.insightsBtn) el.insightsBtn.addEventListener('click', openInsightsModal);
  if (el.closeInsightsModalBtn) el.closeInsightsModalBtn.addEventListener('click', closeInsightsModal);

  // Global search triggers
  if (el.globalSearchInput) {
    el.globalSearchInput.addEventListener('focus', () => {
      openSearchModal();
      el.globalSearchInput.blur();
    });
  }

  // Search modal
  if (el.closeSearchModalBtn) el.closeSearchModalBtn.addEventListener('click', closeSearchModal);
  if (el.modalSearchInput) {
    let debounceTimer;
    el.modalSearchInput.addEventListener('input', (e) => {
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        executeSearch(e.target.value.trim());
      }, 250);
    });
  }

  // Keyboard Shortcuts (R-022)
  window.addEventListener('keydown', (e) => {
    // Cmd+K or Ctrl+K
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      openSearchModal();
    }
    // / to focus search if not inside input
    if (e.key === '/' && document.activeElement.tagName !== 'INPUT' && document.activeElement.tagName !== 'TEXTAREA') {
      e.preventDefault();
      openSearchModal();
    }
    // Esc closes modals and detail view
    if (e.key === 'Escape') {
      if (el.searchModal && el.searchModal.classList.contains('active')) {
        closeSearchModal();
      } else if (el.insightsModal && el.insightsModal.classList.contains('active')) {
        closeInsightsModal();
      } else if (state.currentView === 'detail') {
        showDashboard();
      }
    }
  });

  // Filters & Sorting (Immediate reactivity)
  if (el.workspaceSelect) {
    el.workspaceSelect.addEventListener('change', (e) => {
      const val = e.target.value;
      state.filterWorkspace = val;
      if (state.groupBy === 'project') {
        state.drilledProject = val || null;
      }
      loadSessions();
    });
  }

  if (el.sortBySelect) {
    el.sortBySelect.addEventListener('change', (e) => {
      const val = e.target.value;
      if (val === 'date_asc') {
        state.sortBy = 'date';
        state.sortOrder = 'asc';
      } else {
        state.sortBy = val;
        state.sortOrder = 'desc';
      }
      loadSessions();
    });
  }

  const handleDateChange = () => {
    state.dateFrom = el.dateFromInput ? el.dateFromInput.value : '';
    state.dateTo = el.dateToInput ? el.dateToInput.value : '';
    el.datePresets.forEach(p => p.classList.remove('active'));
    loadSessions();
  };

  if (el.dateFromInput) {
    el.dateFromInput.addEventListener('change', handleDateChange);
    el.dateFromInput.addEventListener('input', handleDateChange);
  }

  if (el.dateToInput) {
    el.dateToInput.addEventListener('change', handleDateChange);
    el.dateToInput.addEventListener('input', handleDateChange);
  }

  // Date Presets
  el.datePresets.forEach(preset => {
    preset.addEventListener('click', () => {
      el.datePresets.forEach(p => p.classList.remove('active'));
      preset.classList.add('active');
      applyDatePreset(preset.dataset.preset);
    });
  });

  // Message Filter Chips
  el.msgFilterChips.forEach(chip => {
    chip.addEventListener('click', () => {
      el.msgFilterChips.forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      state.messageFilter = chip.dataset.filter;
      renderTimelineMessages();
    });
  });

  // Detail View Back Button
  if (el.detailBackBtn) {
    el.detailBackBtn.addEventListener('click', showDashboard);
  }

  // Export Buttons (R-021)
  if (el.exportMdBtn) {
    el.exportMdBtn.addEventListener('click', () => exportCurrentSession('markdown'));
  }
  if (el.exportJsonBtn) {
    el.exportJsonBtn.addEventListener('click', () => exportCurrentSession('json'));
  }

  // Group By Segmented Switcher (R-028, R-029)
  if (el.groupByBtns) {
    el.groupByBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        el.groupByBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        state.groupBy = btn.dataset.group;
        state.drilledProject = null;
        localStorage.setItem('se_group_by', state.groupBy);
        renderView();
      });
    });
  }

  // Brand button resets drilldown and returns to main view
  const brandBtn = document.getElementById('brandBtn');
  if (brandBtn) {
    brandBtn.addEventListener('click', () => {
      state.drilledProject = null;
      state.filterWorkspace = '';
      if (el.workspaceSelect) el.workspaceSelect.value = '';
      showDashboard();
      loadSessions();
    });
  }
}

// Local date helper to prevent timezone skew (e.g. UTC shifting into tomorrow)
function formatLocalDate(d) {
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// Date Presets Logic (R-010)
function applyDatePreset(preset) {
  const now = new Date();
  let from = '';
  let to = '';

  switch (preset) {
    case 'today':
      from = formatLocalDate(now);
      to = formatLocalDate(now);
      break;
    case '7d': {
      const past = new Date(now);
      past.setDate(past.getDate() - 7);
      from = formatLocalDate(past);
      to = formatLocalDate(now);
      break;
    }
    case '30d': {
      const past = new Date(now);
      past.setDate(past.getDate() - 30);
      from = formatLocalDate(past);
      to = formatLocalDate(now);
      break;
    }
    case 'all':
      from = '';
      to = '';
      break;
  }

  state.dateFrom = from;
  state.dateTo = to;
  if (el.dateFromInput) el.dateFromInput.value = from;
  if (el.dateToInput) el.dateToInput.value = to;
  loadSessions();
}

// API Calls
async function loadSessions() {
  try {
    const params = new URLSearchParams();
    const effectiveWorkspace = state.filterWorkspace || (state.groupBy === 'project' && state.drilledProject ? state.drilledProject : '');
    if (effectiveWorkspace) params.append('workspace', effectiveWorkspace);
    if (state.sortBy) params.append('sort_by', state.sortBy);
    if (state.sortOrder) params.append('sort_order', state.sortOrder);
    if (state.dateFrom) params.append('from', state.dateFrom);
    if (state.dateTo) params.append('to', state.dateTo);

    const res = await fetch(`/api/sessions?${params.toString()}`);
    if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
    const data = await res.json();

    state.sessions = data.sessions || [];
    state.stats = data.stats || null;

    renderStats();
    renderWorkspaceOptions();
    renderView();
  } catch (err) {
    console.error('Failed to load sessions:', err);
    showToast('❌ Error loading sessions');
  }
}

async function loadSessionDetail(sessionId) {
  try {
    const res = await fetch(`/api/sessions/${sessionId}`);
    if (!res.ok) throw new Error(`HTTP error! status: ${res.status}`);
    const data = await res.json();

    state.selectedSessionId = sessionId;
    state.selectedSessionDetail = data;

    showDetailView(data);
  } catch (err) {
    console.error('Failed to load session detail:', err);
    showToast('❌ Error loading session detail');
  }
}

async function executeSearch(query) {
  if (!query) {
    el.modalSearchResults.innerHTML = '<div class="empty-state"><div class="empty-state-icon">🔍</div><p>Type keywords to search across all prompt transcripts</p></div>';
    return;
  }

  try {
    const params = new URLSearchParams({ q: query });
    const effectiveWorkspace = state.filterWorkspace || (state.groupBy === 'project' && state.drilledProject ? state.drilledProject : '');
    if (effectiveWorkspace) params.append('workspace', effectiveWorkspace);

    const res = await fetch(`/api/search?${params.toString()}`);
    if (!res.ok) throw new Error(`Search error: ${res.status}`);
    const data = await res.json();

    renderSearchResults(data.results || [], query);
  } catch (err) {
    console.error('Search failed:', err);
    el.modalSearchResults.innerHTML = '<div class="empty-state"><p>Search error occurred.</p></div>';
  }
}

// Stats Strip Rendering (R-017)
function renderStats() {
  if (!state.stats || !el.statsStrip) return;
  const s = state.stats;

  el.statsStrip.innerHTML = `
    <div class="stat-card">
      <div class="stat-icon" style="background: rgba(99, 102, 241, 0.15); color: #818cf8;">📂</div>
      <div class="stat-content">
        <span class="stat-val">${s.total_sessions || 0}</span>
        <span class="stat-lbl">Total Sessions</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon" style="background: rgba(168, 85, 247, 0.15); color: #c084fc;">👤</div>
      <div class="stat-content">
        <span class="stat-val">${s.total_user_messages || 0}</span>
        <span class="stat-lbl">User Prompts</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon" style="background: rgba(6, 182, 212, 0.15); color: #22d3ee;">🤖</div>
      <div class="stat-content">
        <span class="stat-val">${s.total_agent_responses || 0}</span>
        <span class="stat-lbl">Agent Responses</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon" style="background: rgba(16, 185, 129, 0.15); color: #34d399;">⚡</div>
      <div class="stat-content">
        <span class="stat-val">${s.total_tool_calls || 0}</span>
        <span class="stat-lbl">Tool Calls</span>
      </div>
    </div>
    <div class="stat-card">
      <div class="stat-icon" style="background: rgba(245, 158, 11, 0.15); color: #fbbf24;">📁</div>
      <div class="stat-content">
        <span class="stat-val">${(s.projects || []).length || (s.workspaces || []).length}</span>
        <span class="stat-lbl">Repositories</span>
      </div>
    </div>
  `;
}

// Workspace Filter & Strip (R-011, R-028)
function renderWorkspaceOptions() {
  if (!state.stats || !state.stats.workspaces) return;
  const workspaces = state.stats.workspaces;
  const currentVal = state.filterWorkspace || (state.groupBy === 'project' && state.drilledProject ? state.drilledProject : '') || '';

  // Populate Dropdown if empty
  if (el.workspaceSelect) {
    if (el.workspaceSelect.options.length <= 1) {
      let html = '<option value="">All Workspaces / Repositories</option>';
      workspaces.forEach(ws => {
        html += `<option value="${escapeHtml(ws)}">${escapeHtml(ws)}</option>`;
      });
      el.workspaceSelect.innerHTML = html;
    }
    el.workspaceSelect.value = currentVal;
  }

  // Populate Strip Pills
  if (el.workspaceStrip) {
    let stripHtml = `<button class="chip ${currentVal === '' ? 'active' : ''}" data-ws="">All (${state.stats.total_sessions || 0})</button>`;
    workspaces.slice(0, 10).forEach(ws => {
      const active = (currentVal === ws) ? 'active' : '';
      stripHtml += `<button class="chip ${active}" data-ws="${escapeHtml(ws)}">${escapeHtml(ws)}</button>`;
    });
    el.workspaceStrip.innerHTML = stripHtml;

    el.workspaceStrip.querySelectorAll('.chip').forEach(btn => {
      btn.addEventListener('click', () => {
        state.filterWorkspace = btn.dataset.ws;
        if (state.groupBy === 'project') {
          state.drilledProject = btn.dataset.ws || null;
        }
        if (el.workspaceSelect) el.workspaceSelect.value = state.filterWorkspace;
        loadSessions();
      });
    });
  }
}

// Multi-mode View Dispatcher (R-028, R-029)
function renderView() {
  if (state.currentView === 'detail') return;

  // Sync Group By control button state
  if (el.groupByBtns) {
    el.groupByBtns.forEach(b => {
      b.classList.toggle('active', b.dataset.group === state.groupBy);
    });
  }

  switch (state.groupBy) {
    case 'project':
      renderProjectHub();
      break;
    case 'date':
      renderDateGroupedView();
      break;
    case 'flat':
    default:
      renderFlatGridView();
      break;
  }
}

// Mode 1: Repository / Project Clustered Hub (R-028)
function renderProjectHub() {
  const container = el.viewContainer || el.sessionGrid;
  if (!container) return;

  // Case A: User has drilled down into a specific repository
  if (state.drilledProject) {
    const projName = state.drilledProject;
    const projSessions = state.sessions.filter(s =>
      s.project_name === projName || (s.projects_touched && s.projects_touched.includes(projName))
    );

    const projSummary = (state.stats && state.stats.projects) ?
      state.stats.projects.find(p => p.name === projName) : null;
    const projPath = projSummary ? projSummary.path : `/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/${projName}`;
    const totalSteps = projSessions.reduce((acc, s) => acc + s.step_count, 0);
    const totalPrompts = projSessions.reduce((acc, s) => acc + s.user_message_count, 0);
    const totalTools = projSessions.reduce((acc, s) => acc + s.tool_call_count, 0);

    // Filter and sort projSessions based on state.projectFilters
    function getFilteredAndSortedProjSessions() {
      let list = [...projSessions];

      // 1. In-Project Search (independent from global search)
      const q = (state.projectFilters.search || '').trim().toLowerCase();
      if (q) {
        list = list.filter(s => {
          const idMatch = (s.id || '').toLowerCase().includes(q);
          const promptMatch = (s.first_user_prompt_preview || '').toLowerCase().includes(q);
          const wsMatch = (s.workspace || '').toLowerCase().includes(q);
          const touchedMatch = (s.projects_touched || []).some(pt => pt.toLowerCase().includes(q));
          let toolMatch = false;
          if (s.tool_calls_map) {
            toolMatch = Object.keys(s.tool_calls_map).some(k => k.toLowerCase().includes(q));
          }
          return idMatch || promptMatch || wsMatch || touchedMatch || toolMatch;
        });
      }

      // 2. In-Project Date Filter (inclusive lifetime overlap)
      if (state.projectFilters.dateFrom || state.projectFilters.dateTo) {
        const fromTime = state.projectFilters.dateFrom ? new Date(state.projectFilters.dateFrom + 'T00:00:00Z').getTime() : 0;
        const toTime = state.projectFilters.dateTo ? new Date(state.projectFilters.dateTo + 'T23:59:59.999Z').getTime() : Infinity;

        list = list.filter(s => {
          const cTime = new Date(s.created_at).getTime();
          const lTime = s.last_message_at ? new Date(s.last_message_at).getTime() : cTime;
          return (lTime >= fromTime) && (cTime <= toTime);
        });
      }

      // 3. In-Project Sort
      const pSort = state.projectFilters.sort || 'date_desc';
      list.sort((a, b) => {
        switch (pSort) {
          case 'date_asc':
            return new Date(a.created_at) - new Date(b.created_at);
          case 'steps_desc':
            return (b.step_count || 0) - (a.step_count || 0);
          case 'messages_desc':
            return (b.user_message_count || 0) - (a.user_message_count || 0);
          case 'tools_desc':
            return (b.tool_call_count || 0) - (a.tool_call_count || 0);
          case 'size_desc':
            return (b.transcript_size_bytes || 0) - (a.transcript_size_bytes || 0);
          case 'date_desc':
          default:
            return new Date(b.created_at) - new Date(a.created_at);
        }
      });

      return list;
    }

    const filtered = getFilteredAndSortedProjSessions();
    const hasActiveFilters = Boolean(
      state.projectFilters.search ||
      state.projectFilters.dateFrom ||
      state.projectFilters.dateTo ||
      state.projectFilters.datePreset !== 'all' ||
      state.projectFilters.sort !== 'date_desc'
    );

    // Show Hero Banner & In-Project Session Stack Toolbar
    if (el.projectHeroBanner) {
      el.projectHeroBanner.style.display = 'flex';
      el.projectHeroBanner.innerHTML = `
        <div class="project-hero-top">
          <div class="project-hero-left">
            <button class="back-btn" id="projectHeroBackBtn"><span>←</span> All Repositories</button>
            <div class="project-hero-info">
              <div class="project-hero-title">
                📁 ${escapeHtml(projName)}
                <span class="git-badge">git repo</span>
              </div>
              <div class="project-hero-path">${escapeHtml(projPath)}</div>
            </div>
          </div>
          <div class="project-hero-stats">
            <span class="chip active">📂 ${projSessions.length} Total Sessions</span>
            <span class="chip">👤 ${totalPrompts} Prompts</span>
            <span class="chip">⚡ ${totalTools} Tool Calls</span>
            <span class="chip">🪜 ${totalSteps.toLocaleString()} Steps</span>
          </div>
        </div>

        <!-- Dedicated In-Project Session Stack Toolbar (R-029) -->
        <div class="project-session-toolbar">
          <div class="project-search-box">
            <span class="search-icon">🔍</span>
            <input type="text" id="projectSearchInput" class="project-search-input" placeholder="Search sessions in ${escapeHtml(projName)}..." value="${escapeHtml(state.projectFilters.search || '')}">
            <button class="clear-search-btn" id="clearProjectSearchBtn" title="Clear in-project search" style="display: ${state.projectFilters.search ? 'inline-flex' : 'none'};">✕</button>
          </div>

          <div class="filter-group">
            <span class="filter-label">Sort:</span>
            <select id="projectSortSelect" class="select-custom" aria-label="Sort sessions in this project">
              <option value="date_desc" ${state.projectFilters.sort === 'date_desc' ? 'selected' : ''}>Most Recent</option>
              <option value="date_asc" ${state.projectFilters.sort === 'date_asc' ? 'selected' : ''}>Oldest First</option>
              <option value="steps_desc" ${state.projectFilters.sort === 'steps_desc' ? 'selected' : ''}>Most Steps</option>
              <option value="messages_desc" ${state.projectFilters.sort === 'messages_desc' ? 'selected' : ''}>Most Prompts</option>
              <option value="tools_desc" ${state.projectFilters.sort === 'tools_desc' ? 'selected' : ''}>Most Tool Calls</option>
              <option value="size_desc" ${state.projectFilters.sort === 'size_desc' ? 'selected' : ''}>Largest Size</option>
            </select>
          </div>

          <div class="filter-group project-date-group">
            <span class="filter-label">Date:</span>
            <input type="date" id="projectDateFrom" class="date-input" value="${state.projectFilters.dateFrom || ''}" aria-label="Start date">
            <span style="color: var(--text-muted); font-size: 0.8rem;">to</span>
            <input type="date" id="projectDateTo" class="date-input" value="${state.projectFilters.dateTo || ''}" aria-label="End date">
            <div class="project-date-presets">
              <button class="chip date-preset-chip ${state.projectFilters.datePreset === 'all' ? 'active' : ''}" data-preset="all">All Time</button>
              <button class="chip date-preset-chip ${state.projectFilters.datePreset === 'today' ? 'active' : ''}" data-preset="today">Today</button>
              <button class="chip date-preset-chip ${state.projectFilters.datePreset === '7d' ? 'active' : ''}" data-preset="7d">7 Days</button>
              <button class="chip date-preset-chip ${state.projectFilters.datePreset === '30d' ? 'active' : ''}" data-preset="30d">30 Days</button>
            </div>
          </div>

          <div class="project-toolbar-summary">
            <span class="project-count-badge" id="projectSessionCountBadge">
              Showing ${filtered.length} of ${projSessions.length} session${projSessions.length === 1 ? '' : 's'}
            </span>
            <button class="btn-ghost-sm" id="resetProjectFiltersBtn" style="display: ${hasActiveFilters ? 'inline-block' : 'none'};">Reset Filters</button>
          </div>
        </div>
      `;

      // Back Button Listener
      document.getElementById('projectHeroBackBtn')?.addEventListener('click', () => {
        state.drilledProject = null;
        state.filterWorkspace = '';
        state.projectFilters = { search: '', sort: 'date_desc', dateFrom: '', dateTo: '', datePreset: 'all' };
        if (el.workspaceSelect) el.workspaceSelect.value = '';
        loadSessions();
      });

      // In-Project Search Input Listener (Reactive, maintains input focus)
      const pSearchInput = document.getElementById('projectSearchInput');
      const clearSearchBtn = document.getElementById('clearProjectSearchBtn');

      if (pSearchInput) {
        pSearchInput.addEventListener('input', (e) => {
          state.projectFilters.search = e.target.value;
          if (clearSearchBtn) {
            clearSearchBtn.style.display = state.projectFilters.search ? 'inline-flex' : 'none';
          }
          const updated = getFilteredAndSortedProjSessions();
          renderProjectSessionGrid(updated);
          updateProjectToolbarSummary(updated.length, projSessions.length);
        });
      }

      if (clearSearchBtn) {
        clearSearchBtn.addEventListener('click', () => {
          state.projectFilters.search = '';
          if (pSearchInput) {
            pSearchInput.value = '';
            pSearchInput.focus();
          }
          clearSearchBtn.style.display = 'none';
          const updated = getFilteredAndSortedProjSessions();
          renderProjectSessionGrid(updated);
          updateProjectToolbarSummary(updated.length, projSessions.length);
        });
      }

      // In-Project Sort Select Listener
      document.getElementById('projectSortSelect')?.addEventListener('change', (e) => {
        state.projectFilters.sort = e.target.value;
        const updated = getFilteredAndSortedProjSessions();
        renderProjectSessionGrid(updated);
        updateProjectToolbarSummary(updated.length, projSessions.length);
      });

      // In-Project Date Pickers
      const pDateFrom = document.getElementById('projectDateFrom');
      const pDateTo = document.getElementById('projectDateTo');

      const onDateChange = () => {
        state.projectFilters.dateFrom = pDateFrom?.value || '';
        state.projectFilters.dateTo = pDateTo?.value || '';
        state.projectFilters.datePreset = '';
        document.querySelectorAll('.project-date-presets .date-preset-chip').forEach(c => c.classList.remove('active'));
        const updated = getFilteredAndSortedProjSessions();
        renderProjectSessionGrid(updated);
        updateProjectToolbarSummary(updated.length, projSessions.length);
      };

      pDateFrom?.addEventListener('change', onDateChange);
      pDateTo?.addEventListener('change', onDateChange);
      pDateFrom?.addEventListener('input', onDateChange);
      pDateTo?.addEventListener('input', onDateChange);

      // In-Project Date Preset Chips
      document.querySelectorAll('.project-date-presets .date-preset-chip').forEach(btn => {
        btn.addEventListener('click', () => {
          document.querySelectorAll('.project-date-presets .date-preset-chip').forEach(c => c.classList.remove('active'));
          btn.classList.add('active');
          const preset = btn.dataset.preset;
          state.projectFilters.datePreset = preset;
          const today = new Date();
          if (preset === 'all') {
            state.projectFilters.dateFrom = '';
            state.projectFilters.dateTo = '';
          } else if (preset === 'today') {
            const todayStr = formatLocalDate(today);
            state.projectFilters.dateFrom = todayStr;
            state.projectFilters.dateTo = todayStr;
          } else if (preset === '7d') {
            const past = new Date(today);
            past.setDate(past.getDate() - 7);
            state.projectFilters.dateFrom = formatLocalDate(past);
            state.projectFilters.dateTo = formatLocalDate(today);
          } else if (preset === '30d') {
            const past = new Date(today);
            past.setDate(past.getDate() - 30);
            state.projectFilters.dateFrom = formatLocalDate(past);
            state.projectFilters.dateTo = formatLocalDate(today);
          }
          if (pDateFrom) pDateFrom.value = state.projectFilters.dateFrom;
          if (pDateTo) pDateTo.value = state.projectFilters.dateTo;
          const updated = getFilteredAndSortedProjSessions();
          renderProjectSessionGrid(updated);
          updateProjectToolbarSummary(updated.length, projSessions.length);
        });
      });

      // Reset Project Filters Button
      document.getElementById('resetProjectFiltersBtn')?.addEventListener('click', () => {
        state.projectFilters = { search: '', sort: 'date_desc', dateFrom: '', dateTo: '', datePreset: 'all' };
        renderProjectHub();
      });
    }

    function renderProjectSessionGrid(list) {
      if (list.length === 0) {
        container.innerHTML = `
          <div class="empty-state">
            <div class="empty-state-icon">📂</div>
            <h3>No sessions match project filters in ${escapeHtml(projName)}</h3>
            <p>Try adjusting your search query, sort option, or date filter.</p>
            <button class="btn-pill" id="emptyResetBtn" style="margin-top: 1rem;">Reset Project Filters</button>
          </div>
        `;
        document.getElementById('emptyResetBtn')?.addEventListener('click', () => {
          state.projectFilters = { search: '', sort: 'date_desc', dateFrom: '', dateTo: '', datePreset: 'all' };
          renderProjectHub();
        });
      } else {
        container.innerHTML = `<div class="session-grid">${renderSessionCardsHtml(list)}</div>`;
        attachSessionCardClicks(container);
      }
    }

    function updateProjectToolbarSummary(filteredCount, totalCount) {
      const badge = document.getElementById('projectSessionCountBadge');
      if (badge) {
        badge.textContent = `Showing ${filteredCount} of ${totalCount} session${totalCount === 1 ? '' : 's'}`;
      }
      if (el.sessionCountBadge) {
        el.sessionCountBadge.textContent = `${filteredCount} session${filteredCount === 1 ? '' : 's'} in ${projName}`;
      }
      const resetBtn = document.getElementById('resetProjectFiltersBtn');
      const hasFilters = Boolean(
        state.projectFilters.search ||
        state.projectFilters.dateFrom ||
        state.projectFilters.dateTo ||
        state.projectFilters.datePreset !== 'all' ||
        state.projectFilters.sort !== 'date_desc'
      );
      if (resetBtn) {
        resetBtn.style.display = hasFilters ? 'inline-block' : 'none';
      }
    }

    if (el.sessionCountBadge) {
      el.sessionCountBadge.textContent = `${filtered.length} session${filtered.length === 1 ? '' : 's'} in ${projName}`;
    }

    renderProjectSessionGrid(filtered);
    return;
  }

  // Case B: Clustered Repository Hub Grid
  if (el.projectHeroBanner) {
    el.projectHeroBanner.style.display = 'none';
  }

  const rawProjects = (state.stats && state.stats.projects) ? state.stats.projects : [];

  // Dynamically map sessions matching active filters to projects
  const sessionsByProj = new Map();
  state.sessions.forEach(s => {
    const pName = s.project_name || 'Default';
    if (!sessionsByProj.has(pName)) {
      sessionsByProj.set(pName, []);
    }
    sessionsByProj.get(pName).push(s);
  });

  const filterActive = Boolean(state.dateFrom || state.dateTo || state.filterWorkspace);

  // Combine project metadata with active filtered session counts
  let projects = rawProjects.map(p => {
    const pSessions = sessionsByProj.get(p.name) || [];
    const countInFilter = pSessions.length;
    return {
      ...p,
      display_session_count: filterActive ? countInFilter : p.session_count,
      has_active_sessions: countInFilter > 0,
      filtered_sessions: pSessions
    };
  });

  // Filter by workspace if selected
  if (state.filterWorkspace) {
    projects = projects.filter(p => p.name.toLowerCase().includes(state.filterWorkspace.toLowerCase()));
  }

  // Sort projects dynamically according to active sort criteria
  projects.sort((a, b) => {
    // If date/workspace filter is active, prioritize projects with active sessions
    if (filterActive) {
      if (a.has_active_sessions && !b.has_active_sessions) return -1;
      if (!a.has_active_sessions && b.has_active_sessions) return 1;
    }

    switch (state.sortBy) {
      case 'steps':
        return b.total_steps - a.total_steps;
      case 'messages':
        return b.total_user_messages - a.total_user_messages;
      case 'size':
        return (b.total_steps + b.total_tool_calls) - (a.total_steps + a.total_tool_calls);
      case 'date':
      default:
        if (state.sortOrder === 'asc') {
          return new Date(a.last_active_at || 0) - new Date(b.last_active_at || 0);
        }
        return new Date(b.last_active_at || 0) - new Date(a.last_active_at || 0);
    }
  });

  if (el.sessionCountBadge) {
    const activeCount = projects.filter(p => p.display_session_count > 0).length;
    if (filterActive) {
      el.sessionCountBadge.textContent = `${state.sessions.length} sessions in ${activeCount} active repos`;
    } else {
      el.sessionCountBadge.textContent = `${activeCount} Active Repos (${projects.length} Total)`;
    }
  }

  if (projects.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">📂</div>
        <h3>No repositories found</h3>
        <p>No project repositories matched your current filter.</p>
      </div>
    `;
    return;
  }

  let html = '<div class="project-grid">';
  projects.forEach(p => {
    const relTime = p.last_active_at ? formatRelativeTime(p.last_active_at) : 'No recorded activity';

    let promptsHtml = '';
    if (p.recent_prompts && p.recent_prompts.length > 0) {
      promptsHtml = p.recent_prompts.map(pr => `
        <div class="project-prompt-chip" title="${escapeHtml(pr)}">
          <span class="prompt-icon">💬</span> ${escapeHtml(pr)}
        </div>
      `).join('');
    } else {
      promptsHtml = `<div class="project-prompt-chip" style="opacity: 0.5;">No user prompts recorded</div>`;
    }

    const sessionBadge = filterActive && p.has_active_sessions
      ? `<span class="badge" style="background: rgba(99, 102, 241, 0.2); color: #818cf8; border: 1px solid rgba(99, 102, 241, 0.4); padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.72rem;">✨ ${p.display_session_count} in filter</span>`
      : '';

    html += `
      <div class="project-card ${p.has_active_sessions ? 'active-filter-card' : ''}" data-project="${escapeHtml(p.name)}">
        <div>
          <div class="project-card-top">
            <div class="project-card-title">
              📁 ${escapeHtml(p.name)}
            </div>
            <div style="display: flex; gap: 0.4rem; align-items: center;">
              ${sessionBadge}
              <span class="git-badge">git repo</span>
            </div>
          </div>
          <div class="project-card-path" title="${escapeHtml(p.path)}">${escapeHtml(p.path)}</div>

          <div class="project-prompts-preview" style="margin-top: 0.85rem;">
            ${promptsHtml}
          </div>
        </div>

        <div>
          <div class="card-metrics" style="margin-bottom: 0.75rem;">
            <span class="metric-item" title="Total conversation sessions in this repository">📂 ${p.display_session_count} session${p.display_session_count === 1 ? '' : 's'}</span>
            <span class="metric-item" title="User prompts in this repository">👤 ${p.total_user_messages} prompts</span>
            <span class="metric-item" title="Tool calls invoked in this repository">⚡ ${p.total_tool_calls} tools</span>
            <span class="metric-item" title="Total steps executed">🪜 ${p.total_steps} steps</span>
          </div>
          <div class="project-card-footer">
            <span>🕒 ${relTime}</span>
            <span class="project-explore-btn">Explore Sessions & Prompts →</span>
          </div>
        </div>
      </div>
    `;
  });
  html += '</div>';

  container.innerHTML = html;

  // Project Card Click Handlers
  container.querySelectorAll('.project-card').forEach(card => {
    card.addEventListener('click', () => {
      const proj = card.dataset.project;
      state.drilledProject = proj;
      state.filterWorkspace = proj;
      if (el.workspaceSelect) el.workspaceSelect.value = proj;
      loadSessions();
    });
  });
}

// Mode 2: Date Grouped View (R-029)
function renderDateGroupedView() {
  if (el.projectHeroBanner) {
    el.projectHeroBanner.style.display = 'none';
  }
  const container = el.viewContainer || el.sessionGrid;
  if (!container) return;

  const sessions = state.sessions;
  if (el.sessionCountBadge) {
    el.sessionCountBadge.textContent = `${sessions.length} session${sessions.length === 1 ? '' : 's'}`;
  }

  if (sessions.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">📂</div>
        <h3>No sessions found</h3>
        <p>No Antigravity IDE sessions matched your filters.</p>
      </div>
    `;
    return;
  }

  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const startOfYesterday = new Date(startOfToday);
  startOfYesterday.setDate(startOfYesterday.getDate() - 1);
  const startOfWeek = new Date(startOfToday);
  startOfWeek.setDate(startOfWeek.getDate() - 7);

  const buckets = {
    today: { title: '🌟 Today', sessions: [] },
    yesterday: { title: '⚡ Yesterday', sessions: [] },
    thisWeek: { title: '📅 Past 7 Days', sessions: [] },
    older: { title: '🗓️ Older', sessions: [] }
  };

  sessions.forEach(s => {
    const d = new Date(s.created_at);
    if (d >= startOfToday) {
      buckets.today.sessions.push(s);
    } else if (d >= startOfYesterday) {
      buckets.yesterday.sessions.push(s);
    } else if (d >= startOfWeek) {
      buckets.thisWeek.sessions.push(s);
    } else {
      buckets.older.sessions.push(s);
    }
  });

  let html = '';
  for (const key of ['today', 'yesterday', 'thisWeek', 'older']) {
    const b = buckets[key];
    if (b.sessions.length === 0) continue;

    html += `
      <div class="date-group-section">
        <div class="date-group-header">
          <span>${b.title}</span>
          <span class="date-group-badge">${b.sessions.length} session${b.sessions.length === 1 ? '' : 's'}</span>
        </div>
        <div class="session-grid">
          ${renderSessionCardsHtml(b.sessions)}
        </div>
      </div>
    `;
  }

  container.innerHTML = html;
  attachSessionCardClicks(container);
}

// Mode 3: Flat Grid View (R-008, R-018)
function renderFlatGridView() {
  if (el.projectHeroBanner) {
    el.projectHeroBanner.style.display = 'none';
  }
  const container = el.viewContainer || el.sessionGrid;
  if (!container) return;

  const sessions = state.sessions;
  if (el.sessionCountBadge) {
    el.sessionCountBadge.textContent = `${sessions.length} session${sessions.length === 1 ? '' : 's'}`;
  }

  if (sessions.length === 0) {
    container.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">📂</div>
        <h3>No sessions found</h3>
        <p>No Antigravity IDE sessions matched your filters.</p>
      </div>
    `;
    return;
  }

  container.innerHTML = `<div class="session-grid">${renderSessionCardsHtml(sessions)}</div>`;
  attachSessionCardClicks(container);
}

// Helper: Render session cards HTML
function renderSessionCardsHtml(sessions) {
  return sessions.map(s => {
    const timeStr = formatDateTime(s.created_at);
    const relTime = formatRelativeTime(s.created_at);
    const preview = s.first_user_prompt_preview || 'No explicit user prompt';
    const sizeKB = Math.round((s.transcript_size_bytes || 0) / 1024);

    return `
      <div class="session-card" data-id="${s.id}">
        <div>
          <div class="card-top">
            <span class="workspace-badge" title="${escapeHtml(s.workspace || s.project_name)}">
              📁 ${escapeHtml(s.project_name || 'Default')}
            </span>
            <span class="session-time" title="${timeStr}">${relTime}</span>
          </div>
          <div class="prompt-snippet" title="${escapeHtml(preview)}">
            "${escapeHtml(preview)}"
          </div>
        </div>
        <div class="card-metrics">
          <span class="metric-item" title="Total steps in session">🪜 ${s.step_count}</span>
          <span class="metric-item" title="User prompts">👤 ${s.user_message_count}</span>
          <span class="metric-item" title="Agent responses">🤖 ${s.agent_response_count}</span>
          <span class="metric-item" title="Tool calls">⚡ ${s.tool_call_count}</span>
          <span class="metric-item" style="margin-left: auto;" title="Transcript file size">💾 ${sizeKB}KB</span>
        </div>
      </div>
    `;
  }).join('');
}

// Helper: Attach card clicks
function attachSessionCardClicks(container) {
  container.querySelectorAll('.session-card').forEach(card => {
    card.addEventListener('click', () => {
      loadSessionDetail(card.dataset.id);
    });
  });
}

// Detail View (R-009, R-015, R-016)
function showDetailView(detail) {
  state.currentView = 'detail';
  if (el.dashboardView) el.dashboardView.style.display = 'none';
  if (el.detailView) el.detailView.classList.add('active');

  const s = detail.session;
  if (el.detailSessionId) el.detailSessionId.textContent = s.id;
  if (el.detailWorkspaceBadge) el.detailWorkspaceBadge.textContent = `📁 ${s.project_name || 'Default'}`;
  if (el.detailTime) el.detailTime.textContent = formatDateTime(s.created_at);

  renderTimelineMessages();
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function showDashboard() {
  state.currentView = 'dashboard';
  if (el.detailView) el.detailView.classList.remove('active');
  if (el.dashboardView) el.dashboardView.style.display = 'block';
}

function renderTimelineMessages() {
  if (!el.timelineContainer || !state.selectedSessionDetail) return;
  const messages = state.selectedSessionDetail.messages || [];

  // Filter messages (R-013)
  const filtered = messages.filter(m => {
    if (state.messageFilter === 'ALL') return true;
    if (state.messageFilter === 'USER_INPUT') return m.type === 'USER_INPUT';
    if (state.messageFilter === 'PLANNER_RESPONSE') return m.type === 'PLANNER_RESPONSE';
    if (state.messageFilter === 'TOOL_CALLS') return m.tool_calls && m.tool_calls.length > 0;
    if (state.messageFilter === 'ERROR') return m.status === 'ERROR' || m.type === 'ERROR_MESSAGE';
    return true;
  });

  if (filtered.length === 0) {
    el.timelineContainer.innerHTML = '<div class="empty-state"><p>No messages match the current filter.</p></div>';
    return;
  }

  el.timelineContainer.innerHTML = filtered.map(msg => {
    const isUser = msg.type === 'USER_INPUT';
    const isAgent = msg.type === 'PLANNER_RESPONSE';
    const isToolOutput = !isUser && !isAgent && msg.type !== 'ERROR_MESSAGE';
    const isError = msg.type === 'ERROR_MESSAGE' || msg.status === 'ERROR';

    const authorName = isUser ? 'User' : isAgent ? 'Agent' : isError ? 'Error' : msg.type;
    const authorIcon = isUser ? '👤' : isAgent ? '🤖' : isError ? '⚠️' : '⚙️';
    const timeStr = formatDateTime(msg.created_at);

    // Markdown rendered content for both User and Agent
    let bodyHtml = '';
    let toolOutputHtml = '';

    if (isUser || isAgent) {
      if (msg.content && msg.content.trim()) {
        try {
          bodyHtml = marked.parse(msg.content);
        } catch (e) {
          bodyHtml = `<p style="white-space: pre-wrap;">${escapeHtml(msg.content)}</p>`;
        }
      }
    } else if (isToolOutput) {
      // Render tool outputs in clean collapsible accordion with code block
      const contentLen = msg.content ? msg.content.length : 0;
      const sizeStr = contentLen > 1024 ? `${Math.round(contentLen / 1024)}KB` : `${contentLen}B`;
      toolOutputHtml = `
        <details class="tool-output-accordion">
          <summary class="tool-output-summary">
            <span>⚙️ <strong>${escapeHtml(msg.type)}</strong> output (${sizeStr})</span>
            <span style="font-size: 0.72rem; opacity: 0.7;">Click to toggle output</span>
          </summary>
          <pre class="tool-output-pre"><code>${escapeHtml(msg.content || '')}</code></pre>
        </details>
      `;
    } else if (isError) {
      bodyHtml = `<div class="error-banner" style="padding: 0.75rem; background: rgba(239, 68, 68, 0.1); border: 1px solid rgba(239, 68, 68, 0.3); border-radius: var(--radius-sm); color: #f87171;">⚠️ ${escapeHtml(msg.content || 'Unknown error')}</div>`;
    }

    // Thinking section rendered with Markdown
    let thinkingHtml = '';
    if (msg.has_thinking && msg.thinking) {
      let thinkingParsed = '';
      try {
        thinkingParsed = marked.parse(msg.thinking);
      } catch (_) {
        thinkingParsed = escapeHtml(msg.thinking);
      }
      thinkingHtml = `
        <details class="thinking-accordion">
          <summary class="thinking-summary">🧠 Reasoning / Inner Monologue (${msg.thinking.length} chars)</summary>
          <div class="thinking-content">${thinkingParsed}</div>
        </details>
      `;
    }

    // Tool calls collapsible cards (R-015)
    let toolCallsHtml = '';
    if (msg.tool_calls && msg.tool_calls.length > 0) {
      toolCallsHtml = `
        <div class="tool-calls-container">
          <div style="font-size: 0.78rem; font-weight: 600; color: var(--text-secondary); margin-bottom: 0.5rem;">
            ⚡ Invoked Tools (${msg.tool_calls.length})
          </div>
          ${msg.tool_calls.map(tc => {
            const argsPretty = tc.args ? JSON.stringify(tc.args, null, 2) : (tc.args_preview || '{}');
            return `
              <details class="tool-call-card">
                <summary class="tool-call-header">
                  <span class="tool-call-name">🛠️ ${escapeHtml(tc.name)}</span>
                  <button class="copy-tool-args-btn btn-icon" style="width: 26px; height: 26px; font-size: 0.75rem;" title="Copy Args" data-args="${escapeHtml(argsPretty)}">📋</button>
                </summary>
                <pre class="tool-call-args"><code>${escapeHtml(argsPretty)}</code></pre>
              </details>
            `;
          }).join('')}
        </div>
      `;
    }

    const formattedStep = typeof msg.step_index === 'number' ? msg.step_index.toLocaleString() : (msg.step_index ?? '0');

    return `
      <div class="message-item ${isUser ? 'user' : isAgent ? 'agent' : 'tool-result'}" data-step="${msg.step_index}">
        <div class="timeline-node" title="Step ${formattedStep}">${formattedStep}</div>
        <div class="message-card">
          <div class="message-header">
            <span class="message-author">${authorIcon} ${authorName}</span>
            <div class="message-meta">
              <span>Step ${formattedStep}</span>
              <span>•</span>
              <span title="${timeStr}">${formatRelativeTime(msg.created_at)}</span>
              ${msg.content ? `<button class="copy-msg-btn btn-icon" style="width: 26px; height: 26px; font-size: 0.75rem;" title="Copy Content" data-content="${escapeHtml(msg.content)}">📋</button>` : ''}
            </div>
          </div>
          ${thinkingHtml}
          ${bodyHtml ? `<div class="message-body">${bodyHtml}</div>` : ''}
          ${toolOutputHtml}
          ${toolCallsHtml}
        </div>
      </div>
    `;
  }).join('');

  // Attach copy listeners (R-026)
  el.timelineContainer.querySelectorAll('.copy-msg-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      copyToClipboard(btn.dataset.content, 'Message copied to clipboard!');
    });
  });

  el.timelineContainer.querySelectorAll('.copy-tool-args-btn').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      copyToClipboard(btn.dataset.args, 'Tool arguments copied to clipboard!');
    });
  });
}

// Export Session (R-021)
function exportCurrentSession(format) {
  if (!state.selectedSessionDetail) return;
  const detail = state.selectedSessionDetail;
  const s = detail.session;
  const messages = detail.messages || [];

  if (format === 'json') {
    const dataStr = "data:text/json;charset=utf-8," + encodeURIComponent(JSON.stringify(detail, null, 2));
    const dlAnchor = document.createElement('a');
    dlAnchor.setAttribute("href", dataStr);
    dlAnchor.setAttribute("download", `session-${s.id}.json`);
    dlAnchor.click();
    showToast('Downloaded session JSON!');
  } else if (format === 'markdown') {
    let md = `# Conversation Session: ${s.id}\n\n`;
    md += `- **Date**: ${formatDateTime(s.created_at)}\n`;
    md += `- **Workspace**: ${s.workspace || s.project_name}\n`;
    md += `- **Total Steps**: ${s.step_count}\n`;
    md += `- **User Messages**: ${s.user_message_count}\n`;
    md += `- **Agent Responses**: ${s.agent_response_count}\n\n---\n\n`;

    messages.forEach(m => {
      const isUser = m.type === 'USER_INPUT';
      const author = isUser ? '👤 **User**' : '🤖 **Agent**';
      md += `### ${author} (Step ${m.step_index} • ${formatDateTime(m.created_at)})\n\n`;
      if (m.content) md += `${m.content}\n\n`;
      if (m.tool_calls && m.tool_calls.length > 0) {
        md += `#### Tools Used:\n`;
        m.tool_calls.forEach(tc => {
          md += `- \`${tc.name}\`: \`\`\`json\n${JSON.stringify(tc.args, null, 2)}\n\`\`\`\n`;
        });
        md += `\n`;
      }
      md += `---\n\n`;
    });

    const dataStr = "data:text/markdown;charset=utf-8," + encodeURIComponent(md);
    const dlAnchor = document.createElement('a');
    dlAnchor.setAttribute("href", dataStr);
    dlAnchor.setAttribute("download", `session-${s.id}.md`);
    dlAnchor.click();
    showToast('Downloaded session Markdown!');
  }
}

// Search Modal (R-012, R-022)
function openSearchModal() {
  if (!el.searchModal) return;
  el.searchModal.classList.add('active');
  if (el.modalSearchInput) {
    el.modalSearchInput.value = '';
    setTimeout(() => el.modalSearchInput.focus(), 100);
  }
}

function closeSearchModal() {
  if (!el.searchModal) return;
  el.searchModal.classList.remove('active');
}

function renderSearchResults(results, query) {
  if (!el.modalSearchResults) return;

  if (results.length === 0) {
    el.modalSearchResults.innerHTML = `
      <div class="empty-state">
        <p>No results found for "<strong>${escapeHtml(query)}</strong>"</p>
      </div>
    `;
    return;
  }

  el.modalSearchResults.innerHTML = `
    <div style="font-size: 0.8rem; color: var(--text-muted); margin-bottom: 0.75rem;">
      Found ${results.length} match${results.length === 1 ? '' : 'es'}:
    </div>
    <div class="search-results-list">
      ${results.map(r => {
        return `
          <div class="search-result-item" data-session-id="${r.session_id}">
            <div style="display: flex; justify-content: space-between; font-size: 0.75rem; color: var(--text-muted); margin-bottom: 0.35rem;">
              <span class="workspace-badge" style="font-size: 0.7rem; padding: 0.1rem 0.4rem;">📁 ${escapeHtml(r.workspace || 'Workspace')}</span>
              <span>Step ${r.step_index} • ${formatRelativeTime(r.created_at)}</span>
            </div>
            <div style="font-size: 0.85rem; color: var(--text-primary); line-height: 1.4;">
              ${highlightSearchMatch(r.content_preview, query)}
            </div>
          </div>
        `;
      }).join('')}
    </div>
  `;

  el.modalSearchResults.querySelectorAll('.search-result-item').forEach(item => {
    item.addEventListener('click', () => {
      closeSearchModal();
      loadSessionDetail(item.dataset.sessionId);
    });
  });
}

function highlightSearchMatch(text, query) {
  if (!text || !query) return escapeHtml(text || '');
  const escapedText = escapeHtml(text);
  const regex = new RegExp(`(${escapeRegExp(query)})`, 'gi');
  return escapedText.replace(regex, '<mark style="background: rgba(99, 102, 241, 0.4); color: #fff; padding: 0.1rem 0.25rem; border-radius: 3px;">$1</mark>');
}

// Insights Modal (R-025)
function openInsightsModal() {
  if (!el.insightsModal || !state.stats) return;
  const s = state.stats;

  // Render Top Tools
  if (el.topToolsList) {
    const tools = s.top_tool_calls || [];
    if (tools.length === 0) {
      el.topToolsList.innerHTML = '<p style="color: var(--text-muted);">No tool call data available.</p>';
    } else {
      const maxCount = Math.max(...tools.map(t => t.count), 1);
      el.topToolsList.innerHTML = tools.map(t => {
        const pct = Math.round((t.count / maxCount) * 100);
        return `
          <div style="margin-bottom: 0.85rem;">
            <div style="display: flex; justify-content: space-between; font-size: 0.82rem; margin-bottom: 0.25rem;">
              <span style="font-family: var(--font-mono); color: #38bdf8;">🛠️ ${escapeHtml(t.name)}</span>
              <span style="font-weight: 600;">${t.count} call${t.count === 1 ? '' : 's'}</span>
            </div>
            <div style="background: var(--bg-glass-active); height: 6px; border-radius: 999px; overflow: hidden;">
              <div style="background: var(--accent-gradient); width: ${pct}%; height: 100%;"></div>
            </div>
          </div>
        `;
      }).join('');
    }
  }

  // Render Top Prompts (R-025)
  if (el.topPromptsList) {
    const prompts = s.top_prompts || [];
    if (prompts.length === 0) {
      el.topPromptsList.innerHTML = '<p style="color: var(--text-muted);">No prompt patterns analyzed yet.</p>';
    } else {
      el.topPromptsList.innerHTML = prompts.map(p => {
        return `
          <div style="background: var(--bg-glass); border: 1px solid var(--border-glass); border-radius: var(--radius-sm); padding: 0.75rem 1rem; margin-bottom: 0.75rem; display: flex; align-items: center; justify-content: space-between; gap: 0.75rem;">
            <div style="font-size: 0.85rem; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
              "${escapeHtml(p.prompt)}"
            </div>
            <div style="display: flex; align-items: center; gap: 0.5rem; flex-shrink: 0;">
              <span class="version-pill">${p.count}x</span>
              <button class="btn-icon" style="width: 26px; height: 26px; font-size: 0.75rem;" title="Copy Prompt" onclick="copyToClipboard('${escapeHtml(p.prompt).replace(/'/g, "\\'")}', 'Prompt copied!')">📋</button>
            </div>
          </div>
        `;
      }).join('');
    }
  }

  el.insightsModal.classList.add('active');
}

function closeInsightsModal() {
  if (!el.insightsModal) return;
  el.insightsModal.classList.remove('active');
}

// Helpers
window.copyToClipboard = function(text, successMsg = 'Copied to clipboard!') {
  if (!navigator.clipboard) {
    // Fallback for non-https/legacy
    const ta = document.createElement('textarea');
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
    showToast(successMsg);
    return;
  }
  navigator.clipboard.writeText(text).then(() => {
    showToast(successMsg);
  }).catch(err => {
    console.error('Clipboard copy failed:', err);
  });
};

function formatDateTime(iso) {
  if (!iso) return '—';
  try {
    const d = new Date(iso);
    return d.toLocaleString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  } catch (_) {
    return iso;
  }
}

function formatRelativeTime(iso) {
  if (!iso) return '';
  try {
    const d = new Date(iso);
    const diffSec = Math.floor((Date.now() - d.getTime()) / 1000);
    if (diffSec < 60) return 'just now';
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
    if (diffSec < 604800) return `${Math.floor(diffSec / 86400)}d ago`;
    return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
  } catch (_) {
    return '';
  }
}

function escapeHtml(str) {
  if (!str) return '';
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function escapeRegExp(string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
