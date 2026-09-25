import { useState, useEffect, useMemo, useRef } from 'react';
import {
  Search,
  RefreshCw,
  Cpu,
  Terminal,
  Zap,
  Globe,
  Code,
  Check,
  Copy,
  X,
  ChevronRight,
  Eye,
  AlertCircle,
  FileCode,
  ShieldAlert,
  Brain,
  Sparkles,
  Bot,
  Activity,
  Award,
  ArrowUpDown,
  SlidersHorizontal,
  ArrowUp,
  ArrowDown,
  Clock,
} from 'lucide-react';

export interface AnalyticsSummary {
  total_sessions: number;
  total_steps: number;
  total_tool_calls: number;
  total_omni_calls: number;
  total_lsp_lookups: number;
  estimated_tokens_saved: number;
  workspaces_count: number;
  languages_count: number;
}

export interface ToolBreakdown {
  name: string;
  category: string;
  count: number;
  is_omni: boolean;
  description: string;
}

export interface LanguageTelemetry {
  language: string;
  extension: string;
  files_inspected: number;
  color: string;
  lsp_engine: string;
}

export interface WorkspaceAnalytics {
  workspace: string;
  sessions_count: number;
  tools_count: number;
  languages: string[];
}

export interface SessionSummary {
  id: string;
  created_at: string;
  workspace: string;
  step_count: number;
  tool_count: number;
  omni_tool_count: number;
  error_count: number;
  prompt_preview: string;
  top_tools: string[];
  languages_touched: string[];
}

export interface AnalyticsResponse {
  summary: AnalyticsSummary;
  tools_breakdown: ToolBreakdown[];
  languages_telemetry: LanguageTelemetry[];
  workspaces_breakdown: WorkspaceAnalytics[];
  sessions: SessionSummary[];
}

export interface ToolCallInfo {
  name: string;
  args_preview: string;
  is_omni: boolean;
  omni_category?: string;
}

export interface SessionStepDisplay {
  step_index: number;
  source: string;
  step_type: string;
  status: string;
  created_at: string;
  content: string;
  thinking?: string;
  tool_calls: ToolCallInfo[];
}

export interface SessionDetailResponse {
  session_id: string;
  workspace: string;
  created_at: string;
  total_steps: number;
  total_tools: number;
  omni_tools: number;
  messages: SessionStepDisplay[];
}

type PersonaType = 'architect' | 'developer' | 'auditor';

export default function AgentAnalytics() {
  const [data, setData] = useState<AnalyticsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Persona
  const [persona, setPersona] = useState<PersonaType>('architect');

  // Filters
  const [selectedWorkspace, setSelectedWorkspace] = useState<string>('ALL');
  const [selectedLanguage, setSelectedLanguage] = useState<string>('ALL');
  const [selectedCategory, setSelectedCategory] = useState<string>('ALL');
  const [omniOnly, setOmniOnly] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [sortBy, setSortBy] = useState<'recent' | 'tools' | 'omni' | 'steps'>('recent');

type RefreshInterval = 'off' | '10s' | '30s' | '1m' | '3m' | '5m';

const INTERVAL_MS: Record<RefreshInterval, number | null> = {
  off: null,
  '10s': 10000,
  '30s': 30000,
  '1m': 60000, // 1 min default per user request
  '3m': 180000,
  '5m': 300000,
};

  // Traversal Timeline Inspector State (Session Explorer equivalent)
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [sessionDetail, setSessionDetail] = useState<SessionDetailResponse | null>(null);
  const [loadingDetail, setLoadingDetail] = useState(false);
  const [detailFilter, setDetailFilter] = useState<'ALL' | 'PROMPTS' | 'THOUGHTS' | 'TOOLS' | 'OMNI' | 'ERRORS'>('ALL');
  const [modalSearchTerm, setModalSearchTerm] = useState('');
  const [modalSortOrder, setModalSortOrder] = useState<'asc' | 'desc'>('asc'); // asc = Step 0 -> N, desc = Step N -> 0
  const [selectedModalTool, setSelectedModalTool] = useState<string>('ALL');
  const [expandAll, setExpandAll] = useState(false);
  const [copiedId, setCopiedId] = useState(false);
  const [activeToolTab, setActiveToolTab] = useState<'omni' | 'standard'>('omni');
  const modalScrollRef = useRef<HTMLDivElement>(null);

  // Auto-refresh (Configurable Dynamic Sync, default 1 min)
  const [refreshInterval, setRefreshInterval] = useState<RefreshInterval>('1m');
  const [secondsUntilNext, setSecondsUntilNext] = useState<number>(60);
  const [lastUpdated, setLastUpdated] = useState<string>('');

  // Fetch Analytics data
  const fetchAnalytics = async (showSpinner = true) => {
    if (showSpinner) setLoading(true);
    setError(null);
    try {
      const res = await fetch('/api/analytics');
      if (!res.ok) {
        throw new Error(`Failed to fetch analytics: ${res.statusText}`);
      }
      const json: AnalyticsResponse = await res.json();
      setData(json);
      setLastUpdated(new Date().toLocaleTimeString());
    } catch (err: any) {
      console.error('Analytics fetch error:', err);
      setError(err.message || 'Error connecting to analytics endpoint');
    } finally {
      if (showSpinner) setLoading(false);
    }
  };

  useEffect(() => {
    fetchAnalytics(true);
  }, []);

  // Real-time dynamic sync: ticker countdown & polling
  useEffect(() => {
    const ms = INTERVAL_MS[refreshInterval];
    if (!ms) {
      setSecondsUntilNext(0);
      return;
    }
    const initialSecs = Math.floor(ms / 1000);
    setSecondsUntilNext(initialSecs);

    const ticker = setInterval(() => {
      setSecondsUntilNext((prev) => {
        if (prev <= 1) {
          fetchAnalytics(false);
          return initialSecs;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(ticker);
  }, [refreshInterval]);

  // Fetch Session Detail
  const openSessionDetail = async (id: string) => {
    setActiveSessionId(id);
    setLoadingDetail(true);
    setDetailFilter('ALL');
    setModalSearchTerm('');
    setModalSortOrder('asc');
    setSelectedModalTool('ALL');
    setExpandAll(false);
    try {
      const res = await fetch(`/api/analytics/session/${id}`);
      if (!res.ok) {
        throw new Error(`Failed to load session details: ${res.status}`);
      }
      const json: SessionDetailResponse = await res.json();
      setSessionDetail(json);
    } catch (err: any) {
      console.error('Detail fetch error:', err);
    } finally {
      setLoadingDetail(false);
    }
  };

  const closeSessionDetail = () => {
    setActiveSessionId(null);
    setSessionDetail(null);
  };

  const scrollToModalTop = () => {
    if (modalScrollRef.current) {
      modalScrollRef.current.scrollTo({ top: 0, behavior: 'smooth' });
    }
  };

  const scrollToModalBottom = () => {
    if (modalScrollRef.current) {
      modalScrollRef.current.scrollTo({ top: modalScrollRef.current.scrollHeight, behavior: 'smooth' });
    }
  };

  // Filtered Sessions
  const filteredSessions = useMemo(() => {
    if (!data) return [];
    return data.sessions
      .filter((s) => {
        if (selectedWorkspace !== 'ALL' && s.workspace !== selectedWorkspace) {
          return false;
        }
        if (selectedLanguage !== 'ALL') {
          const matchLang = s.languages_touched.some(
            (l) => l.toLowerCase() === selectedLanguage.toLowerCase()
          );
          if (!matchLang) return false;
        }
        if (omniOnly && s.omni_tool_count === 0) {
          return false;
        }
        if (selectedCategory !== 'ALL') {
          const hasCategory = s.top_tools.some((t) => {
            if (selectedCategory === 'omni') return t.includes('graph') || t.includes('omni');
            if (selectedCategory === 'fs') return t.includes('file') || t.includes('dir');
            if (selectedCategory === 'terminal') return t.includes('command') || t.includes('task');
            return true;
          });
          if (!hasCategory) return false;
        }
        if (searchTerm.trim()) {
          const q = searchTerm.toLowerCase();
          const matchId = s.id.toLowerCase().includes(q);
          const matchPrompt = s.prompt_preview.toLowerCase().includes(q);
          const matchWs = s.workspace.toLowerCase().includes(q);
          const matchTools = s.top_tools.some((t) => t.toLowerCase().includes(q));
          if (!matchId && !matchPrompt && !matchWs && !matchTools) return false;
        }
        return true;
      })
      .sort((a, b) => {
        if (sortBy === 'recent') {
          return b.created_at.localeCompare(a.created_at);
        }
        if (sortBy === 'tools') {
          return b.tool_count - a.tool_count;
        }
        if (sortBy === 'omni') {
          return b.omni_tool_count - a.omni_tool_count;
        }
        if (sortBy === 'steps') {
          return b.step_count - a.step_count;
        }
        return 0;
      });
  }, [data, selectedWorkspace, selectedLanguage, selectedCategory, omniOnly, searchTerm, sortBy]);

  // Dynamically extract tools present in this session
  const modalAvailableTools = useMemo(() => {
    if (!sessionDetail) return [];
    const set = new Set<string>();
    sessionDetail.messages.forEach((m) => {
      m.tool_calls.forEach((tc) => set.add(tc.name));
    });
    return Array.from(set).sort();
  }, [sessionDetail]);

  // Modal Step Counts for Badges
  const modalStepCounts = useMemo(() => {
    if (!sessionDetail) return { all: 0, prompts: 0, thoughts: 0, tools: 0, omni: 0, errors: 0 };
    let prompts = 0;
    let thoughts = 0;
    let tools = 0;
    let omni = 0;
    let errors = 0;
    sessionDetail.messages.forEach((m) => {
      if (m.step_type === 'USER_INPUT' || m.source === 'USER_INPUT') prompts++;
      if (m.thinking && m.thinking.trim().length > 0) thoughts++;
      if (m.tool_calls.length > 0) tools++;
      if (m.tool_calls.some((tc) => tc.is_omni)) omni++;
      if (m.status === 'ERROR') errors++;
    });
    return {
      all: sessionDetail.messages.length,
      prompts,
      thoughts,
      tools,
      omni,
      errors,
    };
  }, [sessionDetail]);

  // Filtered & Sorted Step Messages inside Modal (Session Explorer equivalent)
  const filteredSteps = useMemo(() => {
    if (!sessionDetail) return [];
    let list = sessionDetail.messages.filter((m) => {
      // 1. Step type filter
      if (detailFilter === 'PROMPTS' && m.step_type !== 'USER_INPUT' && m.source !== 'USER_INPUT') {
        return false;
      }
      if (detailFilter === 'THOUGHTS' && (!m.thinking || m.thinking.trim().length === 0)) {
        return false;
      }
      if (detailFilter === 'TOOLS' && m.tool_calls.length === 0) {
        return false;
      }
      if (detailFilter === 'OMNI' && !m.tool_calls.some((tc) => tc.is_omni)) {
        return false;
      }
      if (detailFilter === 'ERRORS' && m.status !== 'ERROR') {
        return false;
      }

      // 2. Specific tool filter
      if (selectedModalTool !== 'ALL' && !m.tool_calls.some((tc) => tc.name === selectedModalTool)) {
        return false;
      }

      // 3. Search query filter (searches prompt, thought, tool name, args)
      if (modalSearchTerm.trim()) {
        const q = modalSearchTerm.toLowerCase();
        const inContent = m.content.toLowerCase().includes(q);
        const inThinking = m.thinking ? m.thinking.toLowerCase().includes(q) : false;
        const inTools = m.tool_calls.some(
          (tc) => tc.name.toLowerCase().includes(q) || tc.args_preview.toLowerCase().includes(q)
        );
        if (!inContent && !inThinking && !inTools) {
          return false;
        }
      }

      return true;
    });

    if (modalSortOrder === 'desc') {
      return [...list].reverse();
    }
    return list;
  }, [sessionDetail, detailFilter, selectedModalTool, modalSearchTerm, modalSortOrder]);

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopiedId(true);
    setTimeout(() => setCopiedId(false), 2000);
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        width: '100%',
        background: '#070a13',
        color: '#f8fafc',
        overflowY: 'auto',
        padding: '24px 32px 64px',
        boxSizing: 'border-box',
      }}
    >
      {/* ─── Top Control Bar: Persona Switcher & Live Refresh ───────────── */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          flexWrap: 'wrap',
          gap: 16,
          marginBottom: 20,
          borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
          paddingBottom: 16,
        }}
      >
        <div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <h1
              style={{
                margin: 0,
                fontSize: '1.45rem',
                fontWeight: 700,
                letterSpacing: '-0.02em',
                background: 'linear-gradient(135deg, #38bdf8 0%, #c084fc 100%)',
                WebkitBackgroundClip: 'text',
                WebkitTextFillColor: 'transparent',
              }}
            >
              Omni-DB Agent Analytics & LSP Telemetry
            </h1>
            <span
              style={{
                fontSize: '0.68rem',
                fontWeight: 700,
                padding: '2px 8px',
                borderRadius: 20,
                background: 'rgba(56, 189, 248, 0.15)',
                color: '#38bdf8',
                border: '1px solid rgba(56, 189, 248, 0.3)',
                letterSpacing: '0.05em',
                textTransform: 'uppercase',
              }}
            >
              Rule 08 Grounding
            </span>
          </div>
          <div style={{ fontSize: '0.8rem', color: '#94a3b8', marginTop: 4 }}>
            Observability and AST telemetry across autonomous coding agent sessions, workspaces, and syntax engines.
          </div>
        </div>

        {/* Persona Tabs & Refresh */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          {/* Persona Pills */}
          <div
            style={{
              display: 'flex',
              background: 'rgba(15, 23, 42, 0.8)',
              padding: 3,
              borderRadius: 8,
              border: '1px solid rgba(255, 255, 255, 0.08)',
            }}
          >
            {[
              { id: 'architect', label: 'Lead AI Architect', icon: Brain },
              { id: 'developer', label: 'Systems Researcher', icon: Code },
              { id: 'auditor', label: 'Token & Cost Auditor', icon: ShieldAlert },
            ].map(({ id, label, icon: Icon }) => (
              <button
                key={id}
                onClick={() => setPersona(id as PersonaType)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  padding: '6px 12px',
                  borderRadius: 6,
                  border: 'none',
                  fontSize: '0.76rem',
                  fontWeight: persona === id ? 600 : 400,
                  cursor: 'pointer',
                  background: persona === id ? 'linear-gradient(135deg, rgba(56, 189, 248, 0.2), rgba(192, 132, 252, 0.2))' : 'transparent',
                  color: persona === id ? '#38bdf8' : '#94a3b8',
                  boxShadow: persona === id ? '0 0 12px rgba(56, 189, 248, 0.2)' : 'none',
                  transition: 'all 0.18s ease',
                }}
              >
                <Icon size={14} color={persona === id ? '#38bdf8' : '#64748b'} />
                <span>{label}</span>
              </button>
            ))}
          </div>

          {/* Dynamic Sync Interval Selector (1 min default per user request) */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 8,
              padding: '5px 12px',
              borderRadius: 8,
              background: refreshInterval !== 'off' ? 'rgba(52, 211, 153, 0.12)' : 'rgba(255, 255, 255, 0.05)',
              border: `1px solid ${refreshInterval !== 'off' ? 'rgba(52, 211, 153, 0.35)' : 'rgba(255, 255, 255, 0.12)'}`,
              transition: 'all 0.18s ease',
            }}
            title="Choose dynamic disk scan interval (default: 1 min)"
          >
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: '50%',
                background: refreshInterval !== 'off' ? '#34d399' : '#64748b',
                boxShadow: refreshInterval !== 'off' ? '0 0 8px #34d399' : 'none',
              }}
              className={refreshInterval !== 'off' ? 'pulsing-dot' : ''}
            />
            <Clock size={12} color={refreshInterval !== 'off' ? '#34d399' : '#64748b'} />
            <span style={{ fontSize: '0.72rem', color: refreshInterval !== 'off' ? '#34d399' : '#94a3b8', fontWeight: 600 }}>
              Live Sync:
            </span>
            <select
              value={refreshInterval}
              onChange={(e) => setRefreshInterval(e.target.value as RefreshInterval)}
              style={{
                background: 'rgba(15, 23, 42, 0.6)',
                border: '1px solid rgba(255, 255, 255, 0.15)',
                borderRadius: 4,
                color: refreshInterval !== 'off' ? '#6ee7b7' : '#94a3b8',
                fontSize: '0.72rem',
                fontWeight: 600,
                cursor: 'pointer',
                padding: '2px 6px',
                outline: 'none',
              }}
            >
              <option value="10s" style={{ background: '#0f172a', color: '#fff' }}>10s</option>
              <option value="30s" style={{ background: '#0f172a', color: '#fff' }}>30s</option>
              <option value="1m" style={{ background: '#0f172a', color: '#fff' }}>1 min (default)</option>
              <option value="3m" style={{ background: '#0f172a', color: '#fff' }}>3 mins</option>
              <option value="5m" style={{ background: '#0f172a', color: '#fff' }}>5 mins</option>
              <option value="off" style={{ background: '#0f172a', color: '#94a3b8' }}>Paused (Off)</option>
            </select>
            {refreshInterval !== 'off' && (
              <span style={{ fontSize: '0.68rem', color: '#38bdf8', fontFamily: 'monospace', minWidth: 28 }}>
                {secondsUntilNext}s
              </span>
            )}
            {lastUpdated && (
              <span style={{ fontSize: '0.66rem', color: '#6ee7b7', marginLeft: 2 }}>
                ({lastUpdated})
              </span>
            )}
          </div>

          <button
            onClick={() => fetchAnalytics(true)}
            disabled={loading}
            className="cyber-button-secondary"
            style={{ padding: '6px 12px', fontSize: '0.78rem' }}
          >
            <RefreshCw size={13} className={loading ? 'pulsing-dot' : ''} />
            <span>{loading ? 'Refreshing...' : 'Refresh'}</span>
          </button>
        </div>
      </div>

      {/* Persona Guidance Banner */}
      <div
        style={{
          background: 'rgba(30, 41, 59, 0.4)',
          borderLeft: '4px solid #38bdf8',
          borderRadius: '0 8px 8px 0',
          padding: '8px 16px',
          marginBottom: 20,
          fontSize: '0.78rem',
          color: '#cbd5e1',
          display: 'flex',
          alignItems: 'center',
          gap: 12,
        }}
      >
        <Sparkles size={16} color="#38bdf8" />
        <div>
          {persona === 'architect' && (
            <span>
              <strong>Lead AI Architect View:</strong> Tracking AST call graph condensation (<code style={{ color: '#34d399' }}>make graph-condense</code>) vs multi-file dumps. Ensuring agents inspect subsystem boundaries before altering interfaces.
            </span>
          )}
          {persona === 'developer' && (
            <span>
              <strong>Systems Researcher View:</strong> Analyzing tool usage frequencies (<code style={{ color: '#38bdf8' }}>view_file</code>, <code style={{ color: '#fbbf24' }}>replace_file_content</code>), touched programming languages, and LSP symbol resolutions.
            </span>
          )}
          {persona === 'auditor' && (
            <span>
              <strong>Token & Cost Auditor View:</strong> Measuring context efficiency. Each AST condensed query saves an average of ~14,500 tokens over full-context file dumps, keeping LLMs strictly grounded.
            </span>
          )}
        </div>
      </div>

      {error && (
        <div
          style={{
            background: 'rgba(239, 68, 68, 0.15)',
            border: '1px solid rgba(239, 68, 68, 0.4)',
            borderRadius: 8,
            padding: '10px 16px',
            marginBottom: 20,
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            color: '#f87171',
            fontSize: '0.8rem',
          }}
        >
          <AlertCircle size={16} />
          <span>{error}</span>
        </div>
      )}

      {/* ─── Hero Metric Ribbon (Scientist Grade Cards) ──────────────────── */}
      {data && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(210px, 1fr))',
            gap: 12,
            marginBottom: 24,
          }}
        >
          {/* Card 1: Active Sessions */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(56, 189, 248, 0.25)',
              position: 'relative',
              overflow: 'hidden',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                ACTIVE TRAJECTORIES
              </span>
              <Bot size={16} color="#38bdf8" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#f8fafc' }}>
              {data.summary.total_sessions}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              Persisted across IDE runs
            </div>
          </div>

          {/* Card 2: Steps Processed */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(168, 85, 247, 0.25)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                AGENT REASONING STEPS
              </span>
              <Activity size={16} color="#a855f7" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#c084fc' }}>
              {data.summary.total_steps.toLocaleString()}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              User prompts + Model turns
            </div>
          </div>

          {/* Card 3: Tool Invocations */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(251, 191, 36, 0.25)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                TOOL INVOCATIONS
              </span>
              <Terminal size={16} color="#fbbf24" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#fbbf24' }}>
              {data.summary.total_tool_calls.toLocaleString()}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              AST, file ops & command execs
            </div>
          </div>

          {/* Card 4: Estimated Tokens Saved */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(52, 211, 153, 0.35)',
              background: 'linear-gradient(135deg, rgba(15, 23, 42, 0.8), rgba(6, 78, 59, 0.25))',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#34d399', fontWeight: 700, letterSpacing: '0.04em' }}>
                ESTIMATED TOKENS SAVED
              </span>
              <Award size={16} color="#34d399" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#34d399' }}>
              ~{(data.summary.estimated_tokens_saved / 1000).toFixed(0)}k
            </div>
            <div style={{ fontSize: '0.7rem', color: '#6ee7b7', marginTop: 4 }}>
              Via AST condensation & LSP lookup
            </div>
          </div>

          {/* Card 5: Omni-Graph AST Calls */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(56, 189, 248, 0.3)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                OMNI-GRAPH AST CALLS
              </span>
              <Zap size={16} color="#38bdf8" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#38bdf8' }}>
              {data.summary.total_omni_calls}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              Symbol, condense & GraphRAG
            </div>
          </div>

          {/* Card 6: Monitored Workspaces */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(255, 255, 255, 0.12)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                MONITORED WORKSPACES
              </span>
              <Globe size={16} color="#60a5fa" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#f8fafc' }}>
              {data.summary.workspaces_count}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              Distinct repos & partitions
            </div>
          </div>

          {/* Card 7: LSP Language Engines */}
          <div
            className="glass-panel"
            style={{
              padding: '14px 18px',
              border: '1px solid rgba(255, 255, 255, 0.12)',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
              <span style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600, letterSpacing: '0.04em' }}>
                LSP SYNTAX ENGINES
              </span>
              <Code size={16} color="#ec4899" />
            </div>
            <div style={{ fontSize: '1.65rem', fontWeight: 700, color: '#f8fafc' }}>
              {data.summary.languages_count}
            </div>
            <div style={{ fontSize: '0.7rem', color: '#64748b', marginTop: 4 }}>
              Go, Rust, Python, TS, etc.
            </div>
          </div>
        </div>
      )}

      {/* ─── Token Savings & Context Efficiency Comparison Bar ───────────── */}
      <div
        className="glass-panel"
        style={{
          padding: '16px 20px',
          marginBottom: 24,
          background: 'rgba(15, 23, 42, 0.65)',
          border: '1px solid rgba(52, 211, 153, 0.25)',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12, flexWrap: 'wrap', gap: 10 }}>
          <div>
            <span style={{ fontSize: '0.85rem', fontWeight: 700, color: '#34d399', letterSpacing: '-0.01em' }}>
              Context Density & Efficiency Paradigm: Omni-Graph vs Raw File Cat
            </span>
            <div style={{ fontSize: '0.72rem', color: '#94a3b8', marginTop: 2 }}>
              Omni-Graph extracts deterministic 2-hop topological call graphs (&lt;1,500 tokens) rather than raw full-file dumps (~15,000 tokens).
            </div>
          </div>
          <span
            style={{
              background: 'rgba(52, 211, 153, 0.15)',
              color: '#34d399',
              padding: '4px 10px',
              borderRadius: 20,
              fontSize: '0.75rem',
              fontWeight: 700,
              border: '1px solid rgba(52, 211, 153, 0.4)',
            }}
          >
            ⚡ 90.7% Context Window Preserved
          </span>
        </div>

        {/* Visual Progress Bar */}
        <div style={{ height: 16, width: '100%', background: 'rgba(255, 255, 255, 0.05)', borderRadius: 8, overflow: 'hidden', display: 'flex' }}>
          <div
            style={{
              width: '9.3%',
              background: 'linear-gradient(90deg, #34d399, #38bdf8)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '0.65rem',
              fontWeight: 700,
              color: '#070a13',
            }}
            title="Omni-Graph AST Slice: ~1,200 tokens"
          >
            AST
          </div>
          <div
            style={{
              width: '90.7%',
              background: 'rgba(244, 63, 94, 0.3)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '0.65rem',
              color: '#fca5a5',
            }}
            title="Wasted Tokens via Raw Multi-file Cat: ~13,800 tokens eliminated"
          >
            Tokens Saved: ~13,800 tokens per symbol inspection
          </div>
        </div>
      </div>

      {/* ─── Grounding Paradigm: How Coding Agents Replace Grep, Cat & Find ── */}
      <div
        className="glass-panel"
        style={{
          padding: '18px 22px',
          marginBottom: 24,
          background: 'rgba(15, 23, 42, 0.7)',
          border: '1px solid rgba(56, 189, 248, 0.35)',
        }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14, flexWrap: 'wrap', gap: 10 }}>
          <div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <Zap size={16} color="#38bdf8" />
              <h3 style={{ margin: 0, fontSize: '0.96rem', fontWeight: 700, color: '#f8fafc' }}>
                The Grounding Paradigm: How Coding Agents Replace Grep, Cat & Find
              </h3>
              <span
                style={{
                  background: 'rgba(56, 189, 248, 0.2)',
                  color: '#38bdf8',
                  fontSize: '0.65rem',
                  fontWeight: 700,
                  padding: '2px 7px',
                  borderRadius: 4,
                }}
              >
                RULE 08 COMPLIANCE
              </span>
            </div>
            <div style={{ fontSize: '0.74rem', color: '#94a3b8', marginTop: 3 }}>
              Traditional LLM coding agents blow their context window with raw file dumps and noisy regexes. Omni-Graph equips agents with 4 deterministic AST retrieval engines:
            </div>
          </div>
          <div style={{ fontSize: '0.72rem', color: '#34d399', fontWeight: 600 }}>
            Average Tokens Saved per Retrieval: <span style={{ fontSize: '0.92rem', fontWeight: 800 }}>~14,500</span> tokens
          </div>
        </div>

        {/* 4 Replacement Cards Grid */}
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(280px, 1fr))', gap: 12 }}>
          {/* Substitution 1: Replaces Cat */}
          <div
            style={{
              background: 'rgba(0, 0, 0, 0.35)',
              border: '1px solid rgba(56, 189, 248, 0.25)',
              borderRadius: 8,
              padding: '12px 14px',
              display: 'flex',
              flexDirection: 'column',
              gap: 6,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.7rem', fontWeight: 700, color: '#f43f5e', textDecoration: 'line-through' }}>
                ❌ view_file / cat (4,000–15,000 tokens)
              </span>
              <span style={{ fontSize: '0.65rem', color: '#34d399', fontWeight: 700, background: 'rgba(52, 211, 153, 0.15)', padding: '1px 5px', borderRadius: 4 }}>
                99% Reduction
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <code style={{ fontSize: '0.76rem', color: '#38bdf8', fontWeight: 700, background: 'rgba(56, 189, 248, 0.15)', padding: '2px 6px', borderRadius: 4 }}>
                make graph-symbol SYM=name
              </code>
            </div>
            <div style={{ fontSize: '0.7rem', color: '#cbd5e1', lineHeight: 1.4 }}>
              <strong>LSP AST Definition Lookup:</strong> Returns exact declaration, parameters, line numbers, and implementation body in &lt;50 tokens without reading the whole file.
            </div>
          </div>

          {/* Substitution 2: Replaces Grep */}
          <div
            style={{
              background: 'rgba(0, 0, 0, 0.35)',
              border: '1px solid rgba(168, 85, 247, 0.25)',
              borderRadius: 8,
              padding: '12px 14px',
              display: 'flex',
              flexDirection: 'column',
              gap: 6,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.7rem', fontWeight: 700, color: '#f43f5e', textDecoration: 'line-through' }}>
                ❌ grep_search / grep (Lexical / Noisy)
              </span>
              <span style={{ fontSize: '0.65rem', color: '#c084fc', fontWeight: 700, background: 'rgba(168, 85, 247, 0.15)', padding: '1px 5px', borderRadius: 4 }}>
                Compiler Precision
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <code style={{ fontSize: '0.76rem', color: '#c084fc', fontWeight: 700, background: 'rgba(168, 85, 247, 0.15)', padding: '2px 6px', borderRadius: 4 }}>
                make graph-references SYM=name
              </code>
            </div>
            <div style={{ fontSize: '0.7rem', color: '#cbd5e1', lineHeight: 1.4 }}>
              <strong>LSP Call Hierarchy & References:</strong> Traces true CALLS, IMPORTS, and IMPLEMENTS graph edges across files rather than regex substring matches.
            </div>
          </div>

          {/* Substitution 3: Replaces Multi-File Cat */}
          <div
            style={{
              background: 'rgba(0, 0, 0, 0.35)',
              border: '1px solid rgba(52, 211, 153, 0.3)',
              borderRadius: 8,
              padding: '12px 14px',
              display: 'flex',
              flexDirection: 'column',
              gap: 6,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.7rem', fontWeight: 700, color: '#f43f5e', textDecoration: 'line-through' }}>
                ❌ Reading 5–10 Files (25k–50k tokens)
              </span>
              <span style={{ fontSize: '0.65rem', color: '#34d399', fontWeight: 700, background: 'rgba(52, 211, 153, 0.15)', padding: '1px 5px', borderRadius: 4 }}>
                &lt;1,500 Tokens
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <code style={{ fontSize: '0.76rem', color: '#34d399', fontWeight: 700, background: 'rgba(52, 211, 153, 0.15)', padding: '2px 6px', borderRadius: 4 }}>
                make graph-condense SYM=root
              </code>
            </div>
            <div style={{ fontSize: '0.7rem', color: '#cbd5e1', lineHeight: 1.4 }}>
              <strong>Topological AST Call Slice:</strong> Slices root symbol, upstream callers, downstream callees, and structs into a single dense markdown payload.
            </div>
          </div>

          {/* Substitution 4: Replaces Blind Find */}
          <div
            style={{
              background: 'rgba(0, 0, 0, 0.35)',
              border: '1px solid rgba(251, 191, 36, 0.25)',
              borderRadius: 8,
              padding: '12px 14px',
              display: 'flex',
              flexDirection: 'column',
              gap: 6,
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <span style={{ fontSize: '0.7rem', fontWeight: 700, color: '#f43f5e', textDecoration: 'line-through' }}>
                ❌ list_dir / find (Blind Directory Walk)
              </span>
              <span style={{ fontSize: '0.65rem', color: '#fbbf24', fontWeight: 700, background: 'rgba(251, 191, 36, 0.15)', padding: '1px 5px', borderRadius: 4 }}>
                Graph-RAG
              </span>
            </div>
            <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
              <code style={{ fontSize: '0.76rem', color: '#fbbf24', fontWeight: 700, background: 'rgba(251, 191, 36, 0.15)', padding: '2px 6px', borderRadius: 4 }}>
                make query-graph Q="question"
              </code>
            </div>
            <div style={{ fontSize: '0.7rem', color: '#cbd5e1', lineHeight: 1.4 }}>
              <strong>Hybrid Graph-RAG Retrieval:</strong> Combines 384-d semantic embedding search (HF TEI) + SurrealDB graph edges + Leiden community galaxy synthesis.
            </div>
          </div>
        </div>
      </div>

      {/* ─── Two-Column Analytical Overview: LSP Languages + Tool Breakdown ── */}
      {data && (
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(420px, 1fr))',
            gap: 20,
            marginBottom: 24,
          }}
        >
          {/* Panel A: LSP Engine & Language Telemetry Grid */}
          <div className="glass-panel" style={{ padding: '18px 20px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 14 }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '0.92rem', fontWeight: 700, color: '#f8fafc' }}>
                  LSP Engine & Language Telemetry
                </h3>
                <div style={{ fontSize: '0.7rem', color: '#94a3b8', marginTop: 2 }}>
                  Languages traversed by agents & corresponding LSP parse engines
                </div>
              </div>
              <Code size={16} color="#38bdf8" />
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 8, maxHeight: 360, overflowY: 'auto', paddingRight: 4 }}>
              {data.languages_telemetry.map((lang) => {
                const totalInspected = data.languages_telemetry.reduce((acc, curr) => acc + curr.files_inspected, 0);
                const percent = totalInspected > 0 ? ((lang.files_inspected / totalInspected) * 100).toFixed(1) : '0';
                const isSelected = selectedLanguage.toLowerCase() === lang.language.toLowerCase();

                return (
                  <div
                    key={lang.language}
                    onClick={() => setSelectedLanguage(isSelected ? 'ALL' : lang.language)}
                    style={{
                      background: isSelected ? 'rgba(56, 189, 248, 0.15)' : 'rgba(0, 0, 0, 0.25)',
                      border: `1px solid ${isSelected ? '#38bdf8' : 'rgba(255, 255, 255, 0.06)'}`,
                      borderRadius: 8,
                      padding: '10px 12px',
                      cursor: 'pointer',
                      transition: 'all 0.18s ease',
                    }}
                  >
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 6 }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                        <span
                          style={{
                            width: 10,
                            height: 10,
                            borderRadius: '50%',
                            background: lang.color,
                            boxShadow: `0 0 8px ${lang.color}`,
                          }}
                        />
                        <span style={{ fontWeight: 600, fontSize: '0.82rem', color: '#f8fafc' }}>
                          {lang.language}
                        </span>
                        <span
                          style={{
                            fontSize: '0.68rem',
                            color: '#94a3b8',
                            background: 'rgba(255, 255, 255, 0.06)',
                            padding: '1px 6px',
                            borderRadius: 4,
                          }}
                        >
                          {lang.extension}
                        </span>
                      </div>
                      <div style={{ textAlign: 'right' }}>
                        <span style={{ fontWeight: 700, fontSize: '0.82rem', color: '#f8fafc' }}>
                          {lang.files_inspected.toLocaleString()}
                        </span>
                        <span style={{ fontSize: '0.68rem', color: '#64748b', marginLeft: 4 }}>
                          ({percent}%)
                        </span>
                      </div>
                    </div>

                    <div style={{ height: 4, width: '100%', background: 'rgba(255, 255, 255, 0.05)', borderRadius: 2, overflow: 'hidden', marginBottom: 6 }}>
                      <div
                        style={{
                          height: '100%',
                          width: `${percent}%`,
                          background: lang.color,
                        }}
                      />
                    </div>

                    <div style={{ fontSize: '0.68rem', color: '#94a3b8', display: 'flex', alignItems: 'center', gap: 6 }}>
                      <Cpu size={11} color="#64748b" />
                      <span>{lang.lsp_engine}</span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* Panel B: Coding Agent Tool Spectrum & Active Omni Retrievals */}
          <div className="glass-panel" style={{ padding: '18px 20px' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12, flexWrap: 'wrap', gap: 8 }}>
              <div>
                <h3 style={{ margin: 0, fontSize: '0.92rem', fontWeight: 700, color: '#f8fafc' }}>
                  Coding Agent Tool Spectrum
                </h3>
                <div style={{ fontSize: '0.7rem', color: '#94a3b8', marginTop: 2 }}>
                  Active Omni-Graph RAG retrievals vs traditional baseline tools
                </div>
              </div>

              {/* Sub-tab Switcher inside Panel B */}
              <div
                style={{
                  display: 'flex',
                  background: 'rgba(0, 0, 0, 0.4)',
                  padding: 2,
                  borderRadius: 6,
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                }}
              >
                <button
                  type="button"
                  onClick={() => setActiveToolTab('omni')}
                  style={{
                    background: activeToolTab === 'omni' ? 'rgba(56, 189, 248, 0.25)' : 'transparent',
                    color: activeToolTab === 'omni' ? '#38bdf8' : '#94a3b8',
                    border: 'none',
                    borderRadius: 4,
                    padding: '3px 8px',
                    fontSize: '0.7rem',
                    fontWeight: activeToolTab === 'omni' ? 700 : 400,
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    gap: 4,
                  }}
                >
                  <Zap size={11} color={activeToolTab === 'omni' ? '#38bdf8' : '#64748b'} />
                  <span>Omni AST ({data.tools_breakdown.filter((t) => t.is_omni).length})</span>
                </button>
                <button
                  type="button"
                  onClick={() => setActiveToolTab('standard')}
                  style={{
                    background: activeToolTab === 'standard' ? 'rgba(255, 255, 255, 0.1)' : 'transparent',
                    color: activeToolTab === 'standard' ? '#fff' : '#94a3b8',
                    border: 'none',
                    borderRadius: 4,
                    padding: '3px 8px',
                    fontSize: '0.7rem',
                    fontWeight: activeToolTab === 'standard' ? 700 : 400,
                    cursor: 'pointer',
                  }}
                >
                  <span>Traditional ({data.tools_breakdown.filter((t) => !t.is_omni).length})</span>
                </button>
              </div>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 8, maxHeight: 360, overflowY: 'auto', paddingRight: 4 }}>
              {data.tools_breakdown
                .filter((t) => (activeToolTab === 'omni' ? t.is_omni : !t.is_omni))
                .map((t) => {
                  const totalTools = data.summary.total_tool_calls;
                  const percent = totalTools > 0 ? ((t.count / totalTools) * 100).toFixed(1) : '0';

                  return (
                    <div
                      key={t.name}
                      style={{
                        background: t.is_omni ? 'rgba(56, 189, 248, 0.12)' : 'rgba(0, 0, 0, 0.25)',
                        border: `1px solid ${t.is_omni ? 'rgba(56, 189, 248, 0.4)' : 'rgba(255, 255, 255, 0.06)'}`,
                        borderRadius: 8,
                        padding: '10px 12px',
                      }}
                    >
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                          {t.is_omni ? (
                            <Zap size={13} color="#38bdf8" />
                          ) : (
                            <FileCode size={13} color="#94a3b8" />
                          )}
                          <span
                            style={{
                              fontWeight: 600,
                              fontSize: '0.82rem',
                              color: t.is_omni ? '#38bdf8' : '#f8fafc',
                              fontFamily: 'monospace',
                            }}
                          >
                            {t.name}
                          </span>
                          {t.is_omni && (
                            <span
                              style={{
                                fontSize: '0.62rem',
                                fontWeight: 700,
                                background: 'rgba(56, 189, 248, 0.25)',
                                color: '#38bdf8',
                                padding: '1px 5px',
                                borderRadius: 4,
                              }}
                            >
                              HIGH-DENSITY
                            </span>
                          )}
                        </div>
                        <div>
                          <span style={{ fontWeight: 700, fontSize: '0.82rem', color: '#f8fafc' }}>
                            {t.count.toLocaleString()}
                          </span>
                          <span style={{ fontSize: '0.68rem', color: '#64748b', marginLeft: 4 }}>
                            ({percent}%)
                          </span>
                        </div>
                      </div>

                      <div style={{ height: 4, width: '100%', background: 'rgba(255, 255, 255, 0.05)', borderRadius: 2, overflow: 'hidden', marginBottom: 4 }}>
                        <div
                          style={{
                            height: '100%',
                            width: `${percent}%`,
                            background: t.is_omni ? 'linear-gradient(90deg, #38bdf8, #34d399)' : 'rgba(148, 163, 184, 0.6)',
                          }}
                        />
                      </div>

                      <div style={{ fontSize: '0.68rem', color: '#94a3b8' }}>
                        {t.description}
                      </div>
                    </div>
                  );
                })}
            </div>
          </div>
        </div>
      )}

      {/* ─── Multi-Faceted Filter & Search Bar ────────────────────────────── */}
      <div
        className="glass-panel"
        style={{
          padding: '14px 18px',
          marginBottom: 20,
          display: 'flex',
          flexDirection: 'column',
          gap: 12,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 12 }}>
          {/* Search Box */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              background: 'rgba(0, 0, 0, 0.45)',
              border: '1px solid rgba(255, 255, 255, 0.12)',
              borderRadius: 8,
              padding: '6px 12px',
              minWidth: 320,
              flex: 1,
            }}
          >
            <Search size={14} color="#64748b" style={{ marginRight: 8 }} />
            <input
              type="text"
              placeholder="Search by prompt, session UUID, touched workspace, or tool..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              style={{
                background: 'transparent',
                border: 'none',
                outline: 'none',
                color: '#fff',
                fontSize: '0.8rem',
                width: '100%',
              }}
            />
            {searchTerm && (
              <button
                onClick={() => setSearchTerm('')}
                style={{ background: 'none', border: 'none', cursor: 'pointer', padding: 0 }}
              >
                <X size={14} color="#64748b" />
              </button>
            )}
          </div>

          {/* Quick Omni-Graph Only Toggle */}
          <button
            onClick={() => setOmniOnly(!omniOnly)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '6px 12px',
              borderRadius: 8,
              fontSize: '0.78rem',
              fontWeight: 600,
              cursor: 'pointer',
              background: omniOnly ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.05)',
              border: `1px solid ${omniOnly ? '#38bdf8' : 'rgba(255, 255, 255, 0.12)'}`,
              color: omniOnly ? '#38bdf8' : '#94a3b8',
              boxShadow: omniOnly ? '0 0 12px rgba(56, 189, 248, 0.25)' : 'none',
            }}
          >
            <Zap size={13} color={omniOnly ? '#38bdf8' : '#64748b'} />
            <span>Has Omni-Graph Calls</span>
          </button>

          {/* Sort Selector */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: '0.76rem', color: '#94a3b8' }}>
            <span>Sort:</span>
            <select
              value={sortBy}
              onChange={(e: any) => setSortBy(e.target.value)}
              style={{
                background: 'rgba(0, 0, 0, 0.45)',
                border: '1px solid rgba(255, 255, 255, 0.12)',
                color: '#fff',
                borderRadius: 6,
                padding: '4px 8px',
                fontSize: '0.76rem',
                outline: 'none',
                cursor: 'pointer',
              }}
            >
              <option value="recent">Most Recent</option>
              <option value="tools">Most Tools</option>
              <option value="omni">Most Omni Invocations</option>
              <option value="steps">Most Steps</option>
            </select>
          </div>

          {/* Reset Filters */}
          {(selectedWorkspace !== 'ALL' || selectedLanguage !== 'ALL' || omniOnly || searchTerm) && (
            <button
              onClick={() => {
                setSelectedWorkspace('ALL');
                setSelectedLanguage('ALL');
                setOmniOnly(false);
                setSearchTerm('');
              }}
              style={{
                background: 'none',
                border: 'none',
                color: '#f43f5e',
                fontSize: '0.75rem',
                cursor: 'pointer',
                textDecoration: 'underline',
              }}
            >
              Reset Filters
            </button>
          )}
        </div>

        {/* Workspace Quick Filter Chips */}
        {data && data.workspaces_breakdown.length > 0 && (
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap', paddingTop: 4 }}>
            <span style={{ fontSize: '0.7rem', color: '#64748b', fontWeight: 600, marginRight: 4 }}>
              WORKSPACES:
            </span>
            <button
              onClick={() => setSelectedWorkspace('ALL')}
              style={{
                background: selectedWorkspace === 'ALL' ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                border: `1px solid ${selectedWorkspace === 'ALL' ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                color: selectedWorkspace === 'ALL' ? '#38bdf8' : '#94a3b8',
                borderRadius: 6,
                padding: '2px 8px',
                fontSize: '0.7rem',
                cursor: 'pointer',
              }}
            >
              All ({data.summary.total_sessions})
            </button>
            {data.workspaces_breakdown.slice(0, 10).map((ws) => (
              <button
                key={ws.workspace}
                onClick={() => setSelectedWorkspace(selectedWorkspace === ws.workspace ? 'ALL' : ws.workspace)}
                style={{
                  background: selectedWorkspace === ws.workspace ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                  border: `1px solid ${selectedWorkspace === ws.workspace ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                  color: selectedWorkspace === ws.workspace ? '#38bdf8' : '#94a3b8',
                  borderRadius: 6,
                  padding: '2px 8px',
                  fontSize: '0.7rem',
                  cursor: 'pointer',
                }}
              >
                📁 {ws.workspace} ({ws.sessions_count})
              </button>
            ))}
          </div>
        )}

        {/* Tool Category Chips */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap', paddingTop: 2 }}>
          <span style={{ fontSize: '0.7rem', color: '#64748b', fontWeight: 600, marginRight: 4 }}>
            TOOL CATEGORY:
          </span>
          {[
            { id: 'ALL', label: 'All Tools' },
            { id: 'omni', label: '⚡ Omni-Graph AST' },
            { id: 'fs', label: '📁 Filesystem' },
            { id: 'terminal', label: '💻 Terminal & Tasks' },
          ].map(({ id, label }) => (
            <button
              key={id}
              onClick={() => setSelectedCategory(id)}
              style={{
                background: selectedCategory === id ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                border: `1px solid ${selectedCategory === id ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                color: selectedCategory === id ? '#38bdf8' : '#94a3b8',
                borderRadius: 6,
                padding: '2px 8px',
                fontSize: '0.7rem',
                cursor: 'pointer',
              }}
            >
              {label}
            </button>
          ))}
        </div>
      </div>

      {/* ─── Interactive Traversal Sessions Stream ────────────────────────── */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 12 }}>
          <div style={{ fontSize: '0.85rem', fontWeight: 700, color: '#f8fafc' }}>
            Agent Traversal Sessions ({filteredSessions.length})
          </div>
          <div style={{ fontSize: '0.72rem', color: '#64748b' }}>
            Click "Inspect Traversal" to expand step-by-step reasoning & tool parameters
          </div>
        </div>

        {loading && !data ? (
          <div
            className="glass-panel"
            style={{
              padding: '60px 20px',
              textAlign: 'center',
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 14,
              color: '#94a3b8',
            }}
          >
            <RefreshCw size={24} className="pulsing-dot" color="#38bdf8" />
            <div style={{ fontSize: '0.9rem', color: '#f8fafc', fontWeight: 600 }}>
              Scanning Traversal Sessions & Telemetry...
            </div>
          </div>
        ) : filteredSessions.length === 0 ? (
          <div
            className="glass-panel"
            style={{
              padding: '40px',
              textAlign: 'center',
              color: '#94a3b8',
              fontSize: '0.85rem',
            }}
          >
            No sessions match the current filter criteria.
          </div>
        ) : (
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(440px, 1fr))', gap: 14 }}>
            {filteredSessions.map((s) => (
              <div
                key={s.id}
                className="glass-panel"
                style={{
                  padding: '16px 18px',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 12,
                  border: s.omni_tool_count > 0 ? '1px solid rgba(56, 189, 248, 0.35)' : '1px solid rgba(255, 255, 255, 0.08)',
                  background: s.omni_tool_count > 0 ? 'rgba(15, 23, 42, 0.85)' : 'rgba(15, 23, 42, 0.6)',
                  transition: 'transform 0.15s ease, border-color 0.15s ease',
                }}
              >
                {/* Header */}
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                      <span
                        style={{
                          background: 'rgba(56, 189, 248, 0.12)',
                          color: '#38bdf8',
                          padding: '2px 8px',
                          borderRadius: 6,
                          fontSize: '0.72rem',
                          fontWeight: 700,
                        }}
                      >
                        📁 {s.workspace}
                      </span>
                      {s.omni_tool_count > 0 && (
                        <span
                          style={{
                            background: 'rgba(52, 211, 153, 0.2)',
                            color: '#34d399',
                            border: '1px solid rgba(52, 211, 153, 0.4)',
                            padding: '1px 6px',
                            borderRadius: 4,
                            fontSize: '0.65rem',
                            fontWeight: 700,
                            display: 'flex',
                            alignItems: 'center',
                            gap: 4,
                          }}
                        >
                          <Zap size={10} />
                          {s.omni_tool_count} Omni Calls
                        </span>
                      )}
                    </div>
                    <div
                      style={{
                        fontSize: '0.7rem',
                        color: '#64748b',
                        fontFamily: 'monospace',
                        marginTop: 4,
                      }}
                    >
                      {s.id}
                    </div>
                  </div>

                  <span style={{ fontSize: '0.68rem', color: '#64748b' }}>
                    {s.created_at ? new Date(s.created_at).toLocaleString() : 'Recent'}
                  </span>
                </div>

                {/* Prompt Preview */}
                <div
                  style={{
                    fontSize: '0.78rem',
                    color: '#e2e8f0',
                    lineHeight: 1.4,
                    background: 'rgba(0, 0, 0, 0.25)',
                    padding: '8px 10px',
                    borderRadius: 6,
                    borderLeft: '3px solid rgba(168, 85, 247, 0.5)',
                  }}
                >
                  "{s.prompt_preview}"
                </div>

                {/* Metrics & Languages */}
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: 8 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.72rem', color: '#94a3b8' }}>
                    <span>
                      <strong style={{ color: '#f8fafc' }}>{s.step_count}</strong> steps
                    </span>
                    <span>•</span>
                    <span>
                      <strong style={{ color: '#fbbf24' }}>{s.tool_count}</strong> tools
                    </span>
                    {s.error_count > 0 && (
                      <>
                        <span>•</span>
                        <span style={{ color: '#f43f5e' }}>
                          <strong>{s.error_count}</strong> errors
                        </span>
                      </>
                    )}
                  </div>

                  {/* Languages Touched */}
                  <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                    {s.languages_touched.slice(0, 4).map((lang) => (
                      <span
                        key={lang}
                        style={{
                          fontSize: '0.62rem',
                          background: 'rgba(255, 255, 255, 0.08)',
                          color: '#cbd5e1',
                          padding: '1px 5px',
                          borderRadius: 4,
                        }}
                      >
                        {lang}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Top Tools Used */}
                <div style={{ display: 'flex', alignItems: 'center', gap: 4, flexWrap: 'wrap' }}>
                  {s.top_tools.map((t) => (
                    <span
                      key={t}
                      style={{
                        fontSize: '0.65rem',
                        fontFamily: 'monospace',
                        background: t.includes('graph') || t.includes('omni') ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                        color: t.includes('graph') || t.includes('omni') ? '#38bdf8' : '#94a3b8',
                        padding: '2px 6px',
                        borderRadius: 4,
                      }}
                    >
                      {t}
                    </span>
                  ))}
                </div>

                {/* Action Button */}
                <button
                  onClick={() => openSessionDetail(s.id)}
                  className="cyber-button-secondary"
                  style={{
                    width: '100%',
                    justifyContent: 'center',
                    padding: '7px 0',
                    fontSize: '0.76rem',
                    background: s.omni_tool_count > 0 ? 'rgba(56, 189, 248, 0.1)' : 'rgba(255, 255, 255, 0.04)',
                    borderColor: s.omni_tool_count > 0 ? 'rgba(56, 189, 248, 0.3)' : 'rgba(255, 255, 255, 0.1)',
                  }}
                >
                  <Eye size={13} color={s.omni_tool_count > 0 ? '#38bdf8' : '#94a3b8'} />
                  <span>Inspect Traversal Timeline</span>
                  <ChevronRight size={13} />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* ─── Deep Session Inspector Modal ─────────────────────────────────── */}
      {activeSessionId && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.85)',
            backdropFilter: 'blur(20px)',
            WebkitBackdropFilter: 'blur(20px)',
            zIndex: 999999,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '70px 24px 24px',
            boxSizing: 'border-box',
          }}
        >
          <div
            className="glass-panel"
            style={{
              width: '100%',
              maxWidth: 960,
              maxHeight: '85vh',
              background: '#0b1121',
              border: '1px solid rgba(56, 189, 248, 0.35)',
              borderRadius: 14,
              display: 'flex',
              flexDirection: 'column',
              boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.95)',
              overflow: 'hidden',
            }}
          >
            {/* Modal Header */}
            <div
              style={{
                padding: '16px 22px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                background: 'rgba(15, 23, 42, 0.8)',
              }}
            >
              <div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                  <span style={{ fontSize: '1rem', fontWeight: 700, color: '#f8fafc' }}>
                    Session Traversal Inspector
                  </span>
                  {sessionDetail && (
                    <span
                      style={{
                        background: 'rgba(56, 189, 248, 0.15)',
                        color: '#38bdf8',
                        padding: '2px 8px',
                        borderRadius: 6,
                        fontSize: '0.72rem',
                        fontWeight: 600,
                      }}
                    >
                      📁 {sessionDetail.workspace.replace(/['"]+/g, '')}
                    </span>
                  )}
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginTop: 4 }}>
                  <span style={{ fontSize: '0.7rem', color: '#94a3b8', fontFamily: 'monospace' }}>
                    {activeSessionId}
                  </span>
                  <button
                    onClick={() => copyToClipboard(activeSessionId)}
                    style={{ background: 'none', border: 'none', cursor: 'pointer', padding: 0 }}
                    title="Copy Session UUID"
                  >
                    {copiedId ? <Check size={12} color="#34d399" /> : <Copy size={12} color="#64748b" />}
                  </button>
                </div>
              </div>

              <button
                onClick={closeSessionDetail}
                style={{
                  background: 'rgba(255, 255, 255, 0.06)',
                  border: '1px solid rgba(255, 255, 255, 0.12)',
                  borderRadius: 8,
                  padding: 6,
                  color: '#94a3b8',
                  cursor: 'pointer',
                }}
              >
                <X size={18} />
              </button>
            </div>

            {/* Modal Subheader & Controls Bar (Session Explorer Style) */}
            <div
              style={{
                padding: '12px 22px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
                background: 'rgba(0, 0, 0, 0.35)',
                display: 'flex',
                flexDirection: 'column',
                gap: 10,
              }}
            >
              {/* Row 1: Telemetry Stats & Quick Actions */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  flexWrap: 'wrap',
                  gap: 12,
                }}
              >
                {sessionDetail && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 12, fontSize: '0.74rem', color: '#94a3b8' }}>
                    <span>
                      Total Steps: <strong style={{ color: '#fff' }}>{sessionDetail.total_steps}</strong>
                    </span>
                    <span>•</span>
                    <span>
                      Tool Invocations: <strong style={{ color: '#fbbf24' }}>{sessionDetail.total_tools}</strong>
                    </span>
                    <span>•</span>
                    <span>
                      Omni Slices: <strong style={{ color: '#38bdf8' }}>{sessionDetail.omni_tools}</strong>
                    </span>
                    <span style={{ color: '#64748b' }}>
                      (Showing <strong style={{ color: '#34d399' }}>{filteredSteps.length}</strong> of {sessionDetail.messages.length})
                    </span>
                  </div>
                )}

                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  {/* Expand / Collapse All */}
                  <button
                    onClick={() => setExpandAll(!expandAll)}
                    style={{
                      background: 'rgba(255, 255, 255, 0.05)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '4px 10px',
                      color: expandAll ? '#38bdf8' : '#94a3b8',
                      fontSize: '0.72rem',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                      fontWeight: 600,
                    }}
                    title="Toggle full text expansion on all thoughts and outputs"
                  >
                    <span>{expandAll ? 'Collapse Details' : 'Expand All Details'}</span>
                  </button>

                  {/* Scroll to Top */}
                  <button
                    onClick={scrollToModalTop}
                    style={{
                      background: 'rgba(255, 255, 255, 0.05)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '4px 8px',
                      color: '#94a3b8',
                      fontSize: '0.72rem',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                    }}
                    title="Scroll to beginning of session"
                  >
                    <ArrowUp size={12} />
                    <span>Top</span>
                  </button>

                  {/* Scroll to Bottom */}
                  <button
                    onClick={scrollToModalBottom}
                    style={{
                      background: 'rgba(255, 255, 255, 0.05)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '4px 8px',
                      color: '#94a3b8',
                      fontSize: '0.72rem',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: 4,
                    }}
                    title="Scroll to latest step in session"
                  >
                    <ArrowDown size={12} />
                    <span>Bottom</span>
                  </button>
                </div>
              </div>

              {/* Row 2: Search, Sort Order & Tool Dropdown */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  flexWrap: 'wrap',
                }}
              >
                {/* Search Bar */}
                <div style={{ position: 'relative', flex: 1, minWidth: 220 }}>
                  <Search size={13} style={{ position: 'absolute', left: 10, top: 9, color: '#64748b' }} />
                  <input
                    type="text"
                    value={modalSearchTerm}
                    onChange={(e) => setModalSearchTerm(e.target.value)}
                    placeholder="Search session (prompt, thought, tool, args)..."
                    style={{
                      width: '100%',
                      background: 'rgba(15, 23, 42, 0.7)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '6px 28px 6px 30px',
                      fontSize: '0.74rem',
                      color: '#f8fafc',
                      outline: 'none',
                      boxSizing: 'border-box',
                    }}
                  />
                  {modalSearchTerm && (
                    <button
                      onClick={() => setModalSearchTerm('')}
                      style={{
                        position: 'absolute',
                        right: 8,
                        top: 7,
                        background: 'none',
                        border: 'none',
                        color: '#94a3b8',
                        cursor: 'pointer',
                        padding: 0,
                      }}
                    >
                      <X size={12} />
                    </button>
                  )}
                </div>

                {/* Sort Order Toggle */}
                <button
                  onClick={() => setModalSortOrder((prev) => (prev === 'asc' ? 'desc' : 'asc'))}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 5,
                    padding: '6px 12px',
                    borderRadius: 6,
                    background: 'rgba(255, 255, 255, 0.05)',
                    border: '1px solid rgba(255, 255, 255, 0.12)',
                    color: modalSortOrder === 'desc' ? '#38bdf8' : '#cbd5e1',
                    fontSize: '0.72rem',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                  title="Toggle Chronological / Reverse Chronological Traversal"
                >
                  <ArrowUpDown size={12} color={modalSortOrder === 'desc' ? '#38bdf8' : '#94a3b8'} />
                  <span>{modalSortOrder === 'asc' ? 'Oldest (#0 ➔ #N)' : 'Newest (#N ➔ #0)'}</span>
                </button>

                {/* Specific Tool Selector */}
                {modalAvailableTools.length > 0 && (
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 6,
                      background: 'rgba(15, 23, 42, 0.7)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '3px 8px',
                    }}
                  >
                    <SlidersHorizontal size={12} color="#64748b" />
                    <select
                      value={selectedModalTool}
                      onChange={(e) => setSelectedModalTool(e.target.value)}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: selectedModalTool !== 'ALL' ? '#38bdf8' : '#cbd5e1',
                        fontSize: '0.72rem',
                        fontWeight: 600,
                        cursor: 'pointer',
                        outline: 'none',
                      }}
                    >
                      <option value="ALL" style={{ background: '#0f172a', color: '#fff' }}>
                        All Tools ({modalAvailableTools.length})
                      </option>
                      {modalAvailableTools.map((tool) => (
                        <option key={tool} value={tool} style={{ background: '#0f172a', color: '#fff' }}>
                          {tool}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
              </div>

              {/* Row 3: Step Filter Badges */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                {[
                  { id: 'ALL', label: 'ALL', count: modalStepCounts.all },
                  { id: 'PROMPTS', label: 'PROMPTS', count: modalStepCounts.prompts },
                  { id: 'THOUGHTS', label: 'THOUGHTS', count: modalStepCounts.thoughts },
                  { id: 'TOOLS', label: 'TOOLS', count: modalStepCounts.tools },
                  { id: 'OMNI', label: 'OMNI AST', count: modalStepCounts.omni },
                  { id: 'ERRORS', label: 'ERRORS', count: modalStepCounts.errors },
                ].map(({ id, label, count }) => (
                  <button
                    key={id}
                    onClick={() => setDetailFilter(id as any)}
                    style={{
                      background: detailFilter === id ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                      border: `1px solid ${detailFilter === id ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)'}`,
                      color: detailFilter === id ? '#38bdf8' : '#94a3b8',
                      borderRadius: 6,
                      padding: '3px 10px',
                      fontSize: '0.68rem',
                      cursor: 'pointer',
                      fontWeight: detailFilter === id ? 700 : 400,
                      display: 'flex',
                      alignItems: 'center',
                      gap: 6,
                    }}
                  >
                    <span>{label}</span>
                    <span
                      style={{
                        fontSize: '0.62rem',
                        padding: '1px 5px',
                        borderRadius: 10,
                        background: detailFilter === id ? '#38bdf8' : 'rgba(255, 255, 255, 0.08)',
                        color: detailFilter === id ? '#0f172a' : '#94a3b8',
                        fontWeight: 700,
                      }}
                    >
                      {count}
                    </span>
                  </button>
                ))}
              </div>
            </div>

            {/* Modal Body / Timeline */}
            <div
              ref={modalScrollRef}
              style={{
                flex: 1,
                overflowY: 'auto',
                padding: '16px 22px',
                display: 'flex',
                flexDirection: 'column',
                gap: 12,
              }}
            >
              {loadingDetail ? (
                <div style={{ textAlign: 'center', padding: '40px', color: '#94a3b8', fontSize: '0.85rem' }}>
                  <RefreshCw size={24} className="pulsing-dot" style={{ margin: '0 auto 12px' }} />
                  <div>Loading full step trajectory...</div>
                </div>
              ) : filteredSteps.length === 0 ? (
                <div style={{ textAlign: 'center', padding: '40px', color: '#94a3b8', fontSize: '0.85rem' }}>
                  No steps match the active filters or search term: <code>"{modalSearchTerm || detailFilter}"</code>
                </div>
              ) : (
                filteredSteps.map((m) => {
                  const isUser = m.source === 'USER_INPUT' || m.step_type === 'USER_INPUT';
                  const hasOmni = m.tool_calls.some((tc) => tc.is_omni);
                  const isError = m.status === 'ERROR';

                  return (
                    <div
                      key={m.step_index}
                      style={{
                        background: isError
                          ? 'rgba(239, 68, 68, 0.06)'
                          : isUser
                          ? 'rgba(56, 189, 248, 0.08)'
                          : 'rgba(15, 23, 42, 0.5)',
                        border: `1px solid ${
                          isError
                            ? 'rgba(239, 68, 68, 0.45)'
                            : hasOmni
                            ? 'rgba(56, 189, 248, 0.4)'
                            : isUser
                            ? 'rgba(56, 189, 248, 0.2)'
                            : 'rgba(255, 255, 255, 0.06)'
                        }`,
                        borderRadius: 8,
                        padding: '12px 14px',
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 8,
                      }}
                    >
                      {/* Step Header */}
                      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                          <span
                            style={{
                              fontSize: '0.68rem',
                              fontFamily: 'monospace',
                              color: '#64748b',
                            }}
                          >
                            #{m.step_index}
                          </span>
                          <span
                            style={{
                              fontSize: '0.68rem',
                              fontWeight: 700,
                              textTransform: 'uppercase',
                              padding: '1px 6px',
                              borderRadius: 4,
                              background: isUser ? 'rgba(56, 189, 248, 0.2)' : 'rgba(192, 132, 252, 0.2)',
                              color: isUser ? '#38bdf8' : '#c084fc',
                            }}
                          >
                            {m.source}
                          </span>
                          {isError && (
                            <span
                              style={{
                                fontSize: '0.62rem',
                                fontWeight: 700,
                                background: 'rgba(239, 68, 68, 0.2)',
                                color: '#f87171',
                                border: '1px solid rgba(239, 68, 68, 0.4)',
                                padding: '1px 5px',
                                borderRadius: 4,
                                display: 'flex',
                                alignItems: 'center',
                                gap: 3,
                              }}
                            >
                              <AlertCircle size={10} />
                              ERROR
                            </span>
                          )}
                          {hasOmni && (
                            <span
                              style={{
                                fontSize: '0.62rem',
                                fontWeight: 700,
                                background: 'rgba(52, 211, 153, 0.2)',
                                color: '#34d399',
                                border: '1px solid rgba(52, 211, 153, 0.4)',
                                padding: '1px 5px',
                                borderRadius: 4,
                                display: 'flex',
                                alignItems: 'center',
                                gap: 3,
                              }}
                            >
                              <Zap size={10} />
                              Omni-Graph Call
                            </span>
                          )}
                        </div>

                        <span style={{ fontSize: '0.65rem', color: '#64748b' }}>
                          {m.created_at ? new Date(m.created_at).toLocaleTimeString() : ''}
                        </span>
                      </div>

                      {/* Content preview if present */}
                      {m.content && m.content.trim().length > 0 && (
                        <div
                          style={{
                            fontSize: '0.78rem',
                            color: '#cbd5e1',
                            lineHeight: 1.45,
                            whiteSpace: 'pre-wrap',
                            maxHeight: expandAll ? 'none' : 180,
                            overflowY: expandAll ? 'visible' : 'auto',
                            background: 'rgba(0, 0, 0, 0.2)',
                            padding: '8px 10px',
                            borderRadius: 6,
                          }}
                        >
                          {m.content}
                        </div>
                      )}

                      {/* Thinking Block */}
                      {m.thinking && (
                        <div
                          style={{
                            background: 'rgba(30, 41, 59, 0.5)',
                            borderLeft: '3px solid #a855f7',
                            padding: '8px 10px',
                            borderRadius: '0 6px 6px 0',
                            fontSize: '0.74rem',
                            color: '#e2e8f0',
                            lineHeight: 1.4,
                            maxHeight: expandAll ? 'none' : 140,
                            overflowY: expandAll ? 'visible' : 'auto',
                          }}
                        >
                          <div style={{ fontSize: '0.65rem', color: '#c084fc', fontWeight: 600, marginBottom: 4, display: 'flex', alignItems: 'center', gap: 4 }}>
                            <Brain size={11} />
                            AGENT REASONING / THOUGHT
                          </div>
                          {m.thinking}
                        </div>
                      )}

                      {/* Tool Calls inside Step */}
                      {m.tool_calls.length > 0 && (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: 6, marginTop: 4 }}>
                          {m.tool_calls.map((tc, idx) => (
                            <div
                              key={idx}
                              style={{
                                background: tc.is_omni ? 'rgba(56, 189, 248, 0.12)' : 'rgba(0, 0, 0, 0.35)',
                                border: `1px solid ${tc.is_omni ? 'rgba(56, 189, 248, 0.4)' : 'rgba(255, 255, 255, 0.08)'}`,
                                borderRadius: 6,
                                padding: '6px 10px',
                                display: 'flex',
                                flexDirection: 'column',
                                gap: 4,
                              }}
                            >
                              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                                  {tc.is_omni ? <Zap size={12} color="#38bdf8" /> : <Terminal size={12} color="#fbbf24" />}
                                  <span style={{ fontSize: '0.74rem', fontWeight: 700, color: tc.is_omni ? '#38bdf8' : '#f8fafc', fontFamily: 'monospace' }}>
                                    {tc.name}
                                  </span>
                                  {tc.omni_category && (
                                    <span style={{ fontSize: '0.62rem', color: '#34d399', background: 'rgba(52, 211, 153, 0.15)', padding: '1px 5px', borderRadius: 4 }}>
                                      {tc.omni_category}
                                    </span>
                                  )}
                                </div>
                              </div>

                              {tc.args_preview && (
                                <div
                                  style={{
                                    fontSize: '0.68rem',
                                    color: '#94a3b8',
                                    fontFamily: 'monospace',
                                    background: 'rgba(0, 0, 0, 0.25)',
                                    padding: '4px 8px',
                                    borderRadius: 4,
                                    overflowX: 'auto',
                                    whiteSpace: 'nowrap',
                                  }}
                                >
                                  {tc.args_preview}
                                </div>
                              )}
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                  );
                })
              )}
            </div>

            {/* Modal Footer */}
            <div
              style={{
                padding: '12px 22px',
                borderTop: '1px solid rgba(255, 255, 255, 0.08)',
                display: 'flex',
                justifyContent: 'flex-end',
                background: 'rgba(15, 23, 42, 0.8)',
              }}
            >
              <button onClick={closeSessionDetail} className="cyber-button-secondary" style={{ fontSize: '0.78rem' }}>
                Close Inspector
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
