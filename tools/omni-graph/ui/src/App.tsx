import React, { useEffect, useState, useRef, useMemo } from 'react';
import { Cosmograph } from '@cosmograph/react';
import {
  Search,
  Database,
  Cpu,
  Layers,
  Copy,
  Check,
  RefreshCw,
  FolderInput,
  Compass,
  ChevronRight,
  ChevronDown,
  Folder,
  Sparkles,
  X,
  Globe,
  Code,
  ArrowUp,
  FolderPlus,
  Maximize2,
  Zap,
} from 'lucide-react';

interface GraphNode {
  id: string;
  label: string;
  kind: string;
  file_path: string;
  language: string;
  line_start?: number;
  line_end?: number;
  text?: string;
  community?: number | null;
  workspace?: string;
  similarity?: number;
}

interface GraphLink {
  id?: string;
  source: string;
  target: string;
  type?: string;
  category?: string;
  workspace?: string;
}

interface WorkspaceInfo {
  workspace: string;
  total_nodes: number;
  languages: string[];
  files: string[];
}

interface HealthResponse {
  status: string;
  services: {
    rust_app: { status: string };
    surrealdb: { status: string };
    tei: { status: string; model?: string };
  };
}

interface DirEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_codebase: boolean;
  languages: string[];
}

interface BrowseResponse {
  current_path: string;
  parent_path?: string | null;
  entries: DirEntry[];
}

const GALAXY_COLORS = [
  '#38bdf8', // Neon Cyan
  '#a855f7', // Vivid Purple
  '#34d399', // Emerald
  '#fb7185', // Rose Pink
  '#fbbf24', // Amber
  '#60a5fa', // Blue
  '#c084fc', // Lilac
  '#2dd4bf', // Teal
  '#f43f5e', // Crimson
  '#818cf8', // Indigo
  '#f97316', // Orange
  '#4ade80', // Mint
];

export default function App() {
  // Graph state
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [links, setLinks] = useState<GraphLink[]>([]);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);

  // Workspaces state
  const [workspaces, setWorkspaces] = useState<WorkspaceInfo[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState<string | null>(null);
  const [isWorkspaceDropdownOpen, setIsWorkspaceDropdownOpen] = useState(false);

  // Search state
  const [searchQuery, setSearchQuery] = useState('');
  const [isSearching, setIsSearching] = useState(false);

  // Galaxy & Clustering state (Left Panel)
  const [isGalaxyDrawerOpen, setIsGalaxyDrawerOpen] = useState(false);
  const [selectedClusterId, setSelectedClusterId] = useState<number | null>(null);
  const [isolatedClusterId, setIsolatedClusterId] = useState<number | null>(null);
  const [clusterSearchTerm, setClusterSearchTerm] = useState('');
  const [selectedKindFilter, setSelectedKindFilter] = useState<string>('all');
  const [isClustering, setIsClustering] = useState(false);
  const [clusterToast, setClusterToast] = useState<string | null>(null);
  const [autoClusterAfterIngest, setAutoClusterAfterIngest] = useState(true);

  // Directory Browser Modal state
  const [isBrowserModalOpen, setIsBrowserModalOpen] = useState(false);
  const [browsePath, setBrowsePath] = useState<string>('/workspace');
  const [parentPath, setParentPath] = useState<string | null>(null);
  const [dirEntries, setDirEntries] = useState<DirEntry[]>([]);
  const [isLoadingDir, setIsLoadingDir] = useState(false);
  const [selectedFolderForIngest, setSelectedFolderForIngest] = useState<string>('/workspace');
  const [customProjectName, setCustomProjectName] = useState('');
  const [isIngesting, setIsIngesting] = useState(false);
  const [ingestNotice, setIngestNotice] = useState<string | null>(null);

  // Context Condenser state (Right Panel)
  const [copied, setCopied] = useState(false);
  const [condensedText, setCondensedText] = useState<string | null>(null);

  const cosmographRef = useRef<any>(null);

  // ─── 1. Data Fetching ──────────────────────────────────────────────────────
  const loadWorkspaces = async () => {
    try {
      const res = await fetch('/api/workspaces');
      if (res.ok) {
        const data: WorkspaceInfo[] = await res.json();
        setWorkspaces(data || []);
      }
    } catch (err) {
      console.warn('Failed to load workspaces', err);
    }
  };

  const loadGraph = async (wsFilter?: string | null) => {
    try {
      const url = wsFilter ? `/api/graph?workspace=${encodeURIComponent(wsFilter)}` : '/api/graph';
      const res = await fetch(url);
      if (res.ok) {
        const gJson = await res.json();
        setNodes(gJson.nodes || []);
        setLinks(gJson.links || []);
      }
    } catch (err) {
      console.warn('Failed to load graph data', err);
    }
  };

  const loadHealth = async () => {
    try {
      const res = await fetch('/api/health');
      if (res.ok) {
        const hJson = await res.json();
        setHealth(hJson);
      }
    } catch (err) {
      console.warn('Failed to load health status', err);
    }
  };

  useEffect(() => {
    loadHealth();
    loadWorkspaces();
    loadGraph(selectedWorkspace);
    const interval = setInterval(loadHealth, 12000);
    return () => clearInterval(interval);
  }, []);

  const handleSelectWorkspace = (ws: string | null) => {
    setSelectedWorkspace(ws);
    setIsWorkspaceDropdownOpen(false);
    setSelectedNode(null);
    setSelectedClusterId(null);
    setIsolatedClusterId(null);
    loadGraph(ws);
  };

  // ─── 2. Community Galaxy Grouping ──────────────────────────────────────────
  const clusters = useMemo(() => {
    const map = new Map<number, GraphNode[]>();
    for (const node of nodes) {
      if (node.community !== undefined && node.community !== null) {
        if (!map.has(node.community)) {
          map.set(node.community, []);
        }
        map.get(node.community)!.push(node);
      }
    }

    const list = Array.from(map.entries()).map(([cid, clusterNodes]) => {
      const paths = clusterNodes.map((n) => {
        const parts = n.file_path.split('/');
        return parts.length > 1 ? parts.slice(0, -1).join('/') : n.file_path;
      });
      const counts: Record<string, number> = {};
      paths.forEach((p) => (counts[p] = (counts[p] || 0) + 1));
      const dominant = Object.entries(counts).sort((a, b) => b[1] - a[1])[0]?.[0] || 'subsystem';

      return {
        id: cid,
        name: `Cluster #${cid}: ${dominant}`,
        node_count: clusterNodes.length,
        nodes: clusterNodes,
      };
    });

    return list.sort((a, b) => b.node_count - a.node_count);
  }, [nodes]);

  // Active display nodes and links (Respecting Galaxy Isolation Mode)
  const displayNodes = useMemo(() => {
    if (isolatedClusterId === null) return nodes;
    return nodes.filter((n) => n.community === isolatedClusterId);
  }, [nodes, isolatedClusterId]);

  const displayLinks = useMemo(() => {
    if (isolatedClusterId === null) return links;
    const isolatedIds = new Set(displayNodes.map((n) => n.id));
    return links.filter((l) => isolatedIds.has(l.source) && isolatedIds.has(l.target));
  }, [links, displayNodes, isolatedClusterId]);

  // ─── 3. Semantic Vector Search ─────────────────────────────────────────────
  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchQuery.trim()) return;

    setIsSearching(true);
    try {
      const params = new URLSearchParams({ q: searchQuery, k: '15' });
      if (selectedWorkspace) params.append('workspace', selectedWorkspace);
      const res = await fetch(`/api/search?${params.toString()}`);
      if (res.ok) {
        const data = await res.json();
        const topResults: GraphNode[] = data.results || [];
        if (topResults.length > 0) {
          const matchIds = new Set(topResults.map((r) => r.id));
          const firstMatch = nodes.find((n) => n.id === topResults[0].id) || topResults[0];
          setSelectedNode(firstMatch);
          if (cosmographRef.current) {
            cosmographRef.current.selectNodes(nodes.filter((n) => matchIds.has(n.id)));
          }
        }
      }
    } catch (err) {
      console.error('Vector search failed', err);
    } finally {
      setIsSearching(false);
    }
  };

  // ─── 4. Galaxy Partitioning & Clustering Engine Trigger ─────────────────────
  const handleRunClustering = async (targetWs?: string | null) => {
    setIsClustering(true);
    const scopeLabel = targetWs ? `workspace '${targetWs}'` : 'Omniverse';
    setClusterToast(`🌌 Partitioning ${scopeLabel} with Louvain/Leiden modularity...`);
    try {
      const payload: { workspace?: string } = {};
      if (targetWs) payload.workspace = targetWs;

      const res = await fetch('/api/cluster', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (res.ok) {
        const data = await res.json();
        setClusterToast(`✨ Partitioned into ${data.total_communities} galaxy clusters!`);
        await loadGraph(selectedWorkspace);
        setIsGalaxyDrawerOpen(true);
        setTimeout(() => setClusterToast(null), 5000);
      } else {
        const err = await res.json().catch(() => ({}));
        setClusterToast(`❌ Clustering error: ${err.error || 'Server error'}`);
        setTimeout(() => setClusterToast(null), 5000);
      }
    } catch (err) {
      console.error('Clustering error', err);
      setClusterToast('❌ Connection error to clustering engine');
      setTimeout(() => setClusterToast(null), 5000);
    } finally {
      setIsClustering(false);
    }
  };

  // ─── 5. Dynamic Directory Traversal & Ingestion ───────────────────────────
  const fetchDirectory = async (pathTarget?: string) => {
    setIsLoadingDir(true);
    try {
      const url = pathTarget ? `/api/browse?path=${encodeURIComponent(pathTarget)}` : '/api/browse';
      const res = await fetch(url);
      if (res.ok) {
        const data: BrowseResponse = await res.json();
        setBrowsePath(data.current_path);
        setParentPath(data.parent_path || null);
        setDirEntries(data.entries || []);
        setSelectedFolderForIngest(data.current_path);

        const folderName = data.current_path.split('/').filter(Boolean).pop() || '';
        setCustomProjectName(folderName);
      }
    } catch (err) {
      console.error('Failed to browse directory', err);
    } finally {
      setIsLoadingDir(false);
    }
  };

  const openBrowserModal = () => {
    setIsBrowserModalOpen(true);
    fetchDirectory(browsePath || '/workspace');
  };

  const handleIngestExecution = async (targetPath: string, projectOverride?: string) => {
    setIsIngesting(true);
    setIngestNotice('1/2: Indexing AST nodes & generating 384-d vector embeddings...');
    try {
      const payload: { path: string; project?: string } = { path: targetPath };
      if (projectOverride?.trim()) {
        payload.project = projectOverride.trim();
      }

      const res = await fetch('/api/ingest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      if (res.ok) {
        const data = await res.json();
        const r = data.result || {};
        const targetWs = r.workspace || customProjectName;

        if (autoClusterAfterIngest) {
          setIngestNotice(`2/2: Ingested ${r.nodes_created || 0} nodes. Computing galaxy modular clusters...`);
          try {
            const clusterRes = await fetch('/api/cluster', {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ workspace: targetWs }),
            });
            if (clusterRes.ok) {
              const cData = await clusterRes.json();
              setIngestNotice(
                `✨ Complete! Ingested ${r.nodes_created || 0} nodes and partitioned into ${cData.total_communities} galaxy clusters!`
              );
            } else {
              setIngestNotice(`✅ Ingested ${r.nodes_created || 0} nodes into workspace '${targetWs}'!`);
            }
          } catch (e) {
            console.warn('Auto-clustering failed', e);
            setIngestNotice(`✅ Ingested ${r.nodes_created || 0} nodes into workspace '${targetWs}'!`);
          }
        } else {
          setIngestNotice(
            `✅ Ingested ${r.nodes_created || 0} nodes & ${r.edges_created || 0} edges into workspace '${targetWs}'!`
          );
        }

        await loadWorkspaces();
        setSelectedWorkspace(targetWs);
        await loadGraph(targetWs);
        setIsGalaxyDrawerOpen(true);

        setTimeout(() => {
          setIsBrowserModalOpen(false);
          setIngestNotice(null);
        }, 2200);
      } else {
        const errJson = await res.json().catch(() => ({}));
        setIngestNotice(`❌ Ingestion failed: ${errJson.message || 'Unknown error'}`);
      }
    } catch (err) {
      setIngestNotice(`❌ Ingestion error: ${String(err)}`);
    } finally {
      setIsIngesting(false);
    }
  };

  // ─── 6. Context Condenser for Coding Agents ────────────────────────────────
  const handleFetchCondensed = async (symbol: string) => {
    try {
      const params = new URLSearchParams({ symbol, hops: '2' });
      if (selectedWorkspace) params.append('workspace', selectedWorkspace);
      const res = await fetch(`/api/condense?${params.toString()}`);
      if (res.ok) {
        const data = await res.json();
        setCondensedText(data.formatted_markdown);
        navigator.clipboard.writeText(data.formatted_markdown);
        setCopied(true);
        setTimeout(() => setCopied(false), 2500);
      }
    } catch (err) {
      console.error('Failed to condense AST', err);
    }
  };

  const handleFocusCluster = (cid: number) => {
    setSelectedClusterId(cid === selectedClusterId ? null : cid);
    const clusterNodes = nodes.filter((n) => n.community === cid);
    if (cosmographRef.current && clusterNodes.length > 0) {
      cosmographRef.current.selectNodes(clusterNodes);
    }
  };

  const handleToggleIsolateGalaxy = (cid: number, e?: React.MouseEvent) => {
    if (e) e.stopPropagation();
    if (isolatedClusterId === cid) {
      setIsolatedClusterId(null);
    } else {
      setIsolatedClusterId(cid);
      setSelectedClusterId(cid);
    }
  };

  const handleCopyClusterArchitecture = (
    c: { id: number; name: string; nodes: GraphNode[] },
    e?: React.MouseEvent
  ) => {
    if (e) e.stopPropagation();
    const lines = [
      `# ${c.name} (Galaxy #${c.id})`,
      `Total Symbols: ${c.nodes.length}`,
      `Workspace: ${selectedWorkspace || 'All'}`,
      `\n## Symbols:`,
      ...c.nodes.map(
        (n) => `- **${n.label}** (${n.kind}) — \`${n.file_path}${n.line_start ? `:${n.line_start}` : ''}\``
      ),
    ];
    navigator.clipboard.writeText(lines.join('\n'));
    setClusterToast(`📋 Copied Galaxy #${c.id} architecture to clipboard!`);
    setTimeout(() => setClusterToast(null), 3000);
  };

  const handleFocusNode = (node: GraphNode) => {
    setSelectedNode(node);
    if (cosmographRef.current) {
      cosmographRef.current.selectNodes([node]);
    }
  };

  const getNodeColor = (node: GraphNode) => {
    if (node.community !== undefined && node.community !== null) {
      return GALAXY_COLORS[Math.abs(node.community) % GALAXY_COLORS.length];
    }
    switch (node.kind) {
      case 'function':
        return '#38bdf8';
      case 'struct':
      case 'class':
        return '#c084fc';
      case 'import':
        return '#94a3b8';
      default:
        return '#60a5fa';
    }
  };

  return (
    <div style={{ position: 'relative', width: '100vw', height: '100vh', overflow: 'hidden', background: 'var(--bg-dark)' }}>
      {/* ─── 1. TOP FLOATING CONTROL BAR ────────────────────────────────────── */}
      <header
        className="glass-panel"
        style={{
          position: 'absolute',
          top: 14,
          left: 14,
          right: 14,
          height: 60,
          zIndex: 100,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 18px',
        }}
      >
        {/* Brand & Workspace Dropdown */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
          <div
            style={{
              width: 34,
              height: 34,
              borderRadius: 8,
              background: 'linear-gradient(135deg, #0284c7, #9333ea)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              boxShadow: '0 0 15px rgba(56, 189, 248, 0.35)',
            }}
          >
            <Compass size={20} color="#fff" />
          </div>

          <div style={{ display: 'flex', flexDirection: 'column' }}>
            <span style={{ fontWeight: 700, fontSize: '0.98rem', letterSpacing: '-0.02em', color: '#f8fafc' }}>
              Omni-Graph
            </span>
            <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
              Semantic AST & Vector Hub
            </span>
          </div>

          {/* Workspace Filter Dropdown */}
          <div style={{ position: 'relative', marginLeft: 8 }}>
            <button
              onClick={() => setIsWorkspaceDropdownOpen(!isWorkspaceDropdownOpen)}
              className="cyber-button-secondary"
              style={{
                borderRadius: 20,
                padding: '5px 12px',
                background: selectedWorkspace ? 'rgba(56, 189, 248, 0.12)' : 'rgba(255, 255, 255, 0.06)',
                borderColor: selectedWorkspace ? 'rgba(56, 189, 248, 0.5)' : 'rgba(255, 255, 255, 0.15)',
              }}
            >
              <Globe size={14} color={selectedWorkspace ? 'var(--accent-cyan)' : 'var(--text-secondary)'} />
              <span style={{ fontWeight: 600, fontSize: '0.78rem' }}>
                {selectedWorkspace ? `📁 ${selectedWorkspace}` : '🌐 All Workspaces (Omniverse)'}
              </span>
              <ChevronDown size={14} color="var(--text-muted)" />
            </button>

            {isWorkspaceDropdownOpen && (
              <div
                className="glass-panel"
                style={{
                  position: 'absolute',
                  top: '120%',
                  left: 0,
                  width: 280,
                  zIndex: 110,
                  padding: 8,
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 4,
                  boxShadow: '0 20px 30px rgba(0, 0, 0, 0.8)',
                }}
              >
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', padding: '4px 8px', fontWeight: 600 }}>
                  SELECT ACTIVE CODEBASE
                </div>
                <button
                  onClick={() => handleSelectWorkspace(null)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '8px 10px',
                    borderRadius: 6,
                    background: selectedWorkspace === null ? 'rgba(56, 189, 248, 0.15)' : 'transparent',
                    border: 'none',
                    color: selectedWorkspace === null ? 'var(--accent-cyan)' : 'var(--text-primary)',
                    cursor: 'pointer',
                    fontSize: '0.8rem',
                    textAlign: 'left',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <Globe size={14} />
                    <span>All Workspaces (Omniverse)</span>
                  </div>
                  {selectedWorkspace === null && <Check size={14} />}
                </button>

                {workspaces.map((ws) => (
                  <button
                    key={ws.workspace}
                    onClick={() => handleSelectWorkspace(ws.workspace)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '8px 10px',
                      borderRadius: 6,
                      background: selectedWorkspace === ws.workspace ? 'rgba(56, 189, 248, 0.15)' : 'transparent',
                      border: 'none',
                      color: selectedWorkspace === ws.workspace ? 'var(--accent-cyan)' : 'var(--text-primary)',
                      cursor: 'pointer',
                      fontSize: '0.8rem',
                      textAlign: 'left',
                    }}
                  >
                    <div>
                      <div style={{ fontWeight: 600 }}>📁 {ws.workspace}</div>
                      <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
                        {ws.total_nodes} nodes • {ws.languages.join(', ')}
                      </div>
                    </div>
                    {selectedWorkspace === ws.workspace && <Check size={14} />}
                  </button>
                ))}

                {workspaces.length === 0 && (
                  <div style={{ padding: '8px 10px', fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                    No codebases ingested yet.
                  </div>
                )}
              </div>
            )}
          </div>
        </div>

        {/* Semantic Search Bar */}
        <form
          onSubmit={handleSearch}
          style={{
            display: 'flex',
            alignItems: 'center',
            background: 'rgba(0, 0, 0, 0.45)',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: 24,
            padding: '4px 14px',
            width: 380,
            transition: 'border-color 0.2s',
          }}
        >
          <Search size={15} color="var(--text-muted)" style={{ marginRight: 8 }} />
          <input
            type="text"
            placeholder={
              selectedWorkspace
                ? `Semantic search in ${selectedWorkspace}...`
                : 'Semantic search across all codebases...'
            }
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: '#fff',
              fontSize: '0.82rem',
              width: '100%',
            }}
          />
          {searchQuery && (
            <button
              type="button"
              onClick={() => {
                setSearchQuery('');
                setSelectedNode(null);
                loadGraph(selectedWorkspace);
              }}
              style={{ background: 'none', border: 'none', cursor: 'pointer', padding: 0 }}
            >
              <X size={14} color="var(--text-muted)" />
            </button>
          )}
          {isSearching && <RefreshCw size={14} className="pulsing-dot" color="var(--accent-cyan)" style={{ marginLeft: 6 }} />}
        </form>

        {/* Action Controls & Health Badges */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          {/* Galaxies & Subsystems Toggle */}
          <button
            onClick={() => setIsGalaxyDrawerOpen(!isGalaxyDrawerOpen)}
            className={`cyber-button-secondary ${isGalaxyDrawerOpen ? 'active' : ''}`}
            title="Toggle Galaxy Subsystems Navigator"
          >
            <Sparkles size={14} color={isGalaxyDrawerOpen ? 'var(--accent-cyan)' : 'var(--accent-purple)'} />
            <span>Galaxies</span>
            <span
              style={{
                background: 'rgba(168, 85, 247, 0.2)',
                color: 'var(--accent-purple)',
                borderRadius: 10,
                padding: '1px 6px',
                fontSize: '0.7rem',
                fontWeight: 700,
              }}
            >
              {clusters.length}
            </span>
          </button>

          {/* Quick Cluster Button if no clusters exist */}
          {clusters.length === 0 && nodes.length > 0 && (
            <button
              onClick={() => handleRunClustering(selectedWorkspace)}
              disabled={isClustering}
              className="cyber-button-purple"
              style={{ padding: '5px 12px', fontSize: '0.78rem' }}
              title="Compute galaxy clusters for this graph"
            >
              {isClustering ? <RefreshCw size={13} className="pulsing-dot" /> : <Zap size={13} />}
              <span>{isClustering ? 'Clustering...' : 'Cluster Universe'}</span>
            </button>
          )}

          {/* Ingest Codebase Button */}
          <button
            onClick={openBrowserModal}
            className="cyber-button"
            title="Open Interactive Codebase Browser"
          >
            <FolderPlus size={15} />
            <span>Ingest Codebase</span>
          </button>

          {/* Status Indicators */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 10,
              borderLeft: '1px solid rgba(255, 255, 255, 0.1)',
              paddingLeft: 14,
            }}
          >
            <div title="SurrealDB v2 Status" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
              <Database size={13} color={health?.services.surrealdb.status === 'ok' ? '#34d399' : '#f87171'} />
              <span style={{ color: 'var(--text-muted)' }}>DB</span>
            </div>
            <div title="HuggingFace TEI Inference" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
              <Cpu size={13} color={health?.services.tei.status === 'ok' ? '#34d399' : '#f87171'} />
              <span style={{ color: 'var(--text-muted)' }}>TEI</span>
            </div>
            <div title="Visible Nodes" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
              <Layers size={13} color="#38bdf8" />
              <span style={{ color: '#38bdf8', fontWeight: 600 }}>{displayNodes.length}</span>
            </div>
          </div>
        </div>
      </header>

      {/* ─── FLOATING ISOLATION MODE BANNER ─────────────────────────────────── */}
      {isolatedClusterId !== null && (
        <div
          className="glass-panel"
          style={{
            position: 'absolute',
            top: 84,
            left: '50%',
            transform: 'translateX(-50%)',
            zIndex: 95,
            padding: '7px 18px',
            display: 'flex',
            alignItems: 'center',
            gap: 14,
            border: '1px solid rgba(168, 85, 247, 0.6)',
            boxShadow: '0 0 25px rgba(168, 85, 247, 0.3)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
            <Sparkles size={16} color="var(--accent-purple)" />
            <span style={{ fontSize: '0.82rem', fontWeight: 600, color: '#fff' }}>
              Isolated Galaxy #{isolatedClusterId} ({displayNodes.length} symbols active)
            </span>
          </div>
          <button
            onClick={() => setIsolatedClusterId(null)}
            className="cyber-button-secondary"
            style={{ padding: '3px 10px', fontSize: '0.72rem' }}
          >
            <X size={13} />
            <span>Exit Isolation (Show Omniverse)</span>
          </button>
        </div>
      )}

      {/* ─── 2. WEBGL COSMOGRAPH 3D CANVAS ──────────────────────────────────── */}
      <div style={{ width: '100%', height: '100%' }}>
        {displayNodes.length > 0 ? (
          <Cosmograph
            ref={cosmographRef}
            nodes={displayNodes}
            links={displayLinks}
            nodeColor={getNodeColor}
            nodeSize={4}
            linkWidth={1}
            linkColor="rgba(255, 255, 255, 0.12)"
            simulationGravity={0.25}
            simulationRepulsion={1.2}
            simulationLinkSpring={0.8}
            simulationLinkDistance={25}
            backgroundColor="#070a13"
            onClick={(node: any) => setSelectedNode(node || null)}
            onMouseMove={(node: any) => setHoveredNode(node || null)}
          />
        ) : (
          <div
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              height: '100%',
              gap: 16,
              color: 'var(--text-secondary)',
            }}
          >
            <Compass size={54} color="var(--accent-cyan)" className="pulsing-dot" />
            <div style={{ fontSize: '1.25rem', fontWeight: 700, color: '#f8fafc', letterSpacing: '-0.02em' }}>
              {selectedWorkspace ? `No nodes in workspace '${selectedWorkspace}'` : 'Omni-Graph Omniverse Ready'}
            </div>
            <div style={{ maxWidth: 440, textAlign: 'center', fontSize: '0.85rem', color: 'var(--text-muted)', lineHeight: 1.5 }}>
              Browse your local directories or select a folder to index AST structures, typed relationships, and 384-d semantic embeddings.
            </div>
            <button onClick={openBrowserModal} className="cyber-button" style={{ marginTop: 8 }}>
              <FolderPlus size={16} />
              <span>Select Codebase to Ingest</span>
            </button>
          </div>
        )}
      </div>

      {/* Floating Canvas Node Hover Tooltip */}
      {hoveredNode && !selectedNode && (
        <div
          className="glass-panel"
          style={{
            position: 'absolute',
            bottom: 24,
            left: isGalaxyDrawerOpen ? 370 : 24,
            zIndex: 80,
            padding: '8px 14px',
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            pointerEvents: 'none',
            transition: 'left 0.25s ease',
          }}
        >
          <div
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: getNodeColor(hoveredNode),
              boxShadow: `0 0 8px ${getNodeColor(hoveredNode)}`,
            }}
          />
          <span style={{ fontWeight: 600, fontSize: '0.8rem', color: '#fff', fontFamily: 'var(--code-font)' }}>
            {hoveredNode.label}
          </span>
          <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>
            {hoveredNode.kind} • {hoveredNode.file_path}
          </span>
        </div>
      )}

      {/* ─── 3. LEFT COLLAPSIBLE DRAWER: GALAXY CLUSTERS NAVIGATOR ─────────── */}
      <aside
        className="glass-panel drawer-left"
        style={{
          position: 'absolute',
          top: 86,
          left: 14,
          bottom: 14,
          width: 360,
          zIndex: 90,
          display: 'flex',
          flexDirection: 'column',
          padding: 16,
          gap: 12,
          transform: isGalaxyDrawerOpen ? 'translateX(0)' : 'translateX(-390px)',
          opacity: isGalaxyDrawerOpen ? 1 : 0,
          pointerEvents: isGalaxyDrawerOpen ? 'all' : 'none',
        }}
      >
        {/* Drawer Header */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div>
            <div style={{ fontWeight: 700, fontSize: '0.95rem', display: 'flex', alignItems: 'center', gap: 6 }}>
              <Sparkles size={16} color="var(--accent-purple)" />
              <span>Galaxy Subsystems</span>
            </div>
            <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
              {selectedWorkspace ? `Workspace: ${selectedWorkspace}` : 'All Indexed Subsystems'}
            </div>
          </div>
          <button
            onClick={() => setIsGalaxyDrawerOpen(false)}
            style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
          >
            <X size={16} />
          </button>
        </div>

        {/* Command Center: Run / Recompute Clustering */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <button
            onClick={() => handleRunClustering(selectedWorkspace)}
            disabled={isClustering}
            className="cyber-button-purple"
            style={{ width: '100%', justifyContent: 'center' }}
            title="Compute or re-partition Louvain modular clusters"
          >
            {isClustering ? <RefreshCw size={14} className="pulsing-dot" /> : <Sparkles size={14} />}
            <span>
              {isClustering
                ? 'Partitioning Subsystems...'
                : selectedWorkspace
                ? `⚡ Re-cluster ${selectedWorkspace}`
                : '⚡ Compute Galaxy Clusters'}
            </span>
          </button>

          {selectedWorkspace && (
            <button
              onClick={() => handleRunClustering(null)}
              disabled={isClustering}
              className="cyber-button-secondary"
              style={{ width: '100%', justifyContent: 'center', fontSize: '0.72rem', padding: '4px 8px' }}
            >
              <Globe size={12} />
              <span>Cluster Whole Omniverse</span>
            </button>
          )}
        </div>

        {/* Cluster Status Toast */}
        {clusterToast && (
          <div
            style={{
              padding: '6px 10px',
              borderRadius: 6,
              background: clusterToast.startsWith('❌') ? 'rgba(244, 63, 94, 0.15)' : 'rgba(168, 85, 247, 0.15)',
              border: `1px solid ${clusterToast.startsWith('❌') ? 'rgba(244, 63, 94, 0.4)' : 'rgba(168, 85, 247, 0.4)'}`,
              fontSize: '0.72rem',
              color: clusterToast.startsWith('❌') ? '#f43f5e' : '#c084fc',
            }}
          >
            {clusterToast}
          </div>
        )}

        {/* Subsystem Metrics Summary Card */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 6,
            background: 'rgba(0, 0, 0, 0.3)',
            borderRadius: 8,
            padding: '8px 10px',
            border: '1px solid rgba(255, 255, 255, 0.06)',
          }}
        >
          <div>
            <div style={{ fontSize: '0.65rem', color: 'var(--text-muted)' }}>GALAXIES</div>
            <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-purple)' }}>{clusters.length}</div>
          </div>
          <div>
            <div style={{ fontSize: '0.65rem', color: 'var(--text-muted)' }}>CLUSTERED NODES</div>
            <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-cyan)' }}>
              {nodes.filter((n) => n.community !== null && n.community !== undefined).length}
              <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)' }}>/{nodes.length}</span>
            </div>
          </div>
        </div>

        {/* Filter input for galaxies & symbols */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            background: 'rgba(0, 0, 0, 0.4)',
            border: '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: 6,
            padding: '4px 8px',
          }}
        >
          <Search size={13} color="var(--text-muted)" style={{ marginRight: 6 }} />
          <input
            type="text"
            placeholder="Filter clusters or symbols..."
            value={clusterSearchTerm}
            onChange={(e) => setClusterSearchTerm(e.target.value)}
            style={{
              background: 'transparent',
              border: 'none',
              outline: 'none',
              color: '#fff',
              fontSize: '0.75rem',
              width: '100%',
            }}
          />
        </div>

        {/* Symbol Kind Filter Chips */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 4, flexWrap: 'wrap' }}>
          {['all', 'function', 'struct', 'class', 'import'].map((k) => (
            <button
              key={k}
              onClick={() => setSelectedKindFilter(k)}
              style={{
                background: selectedKindFilter === k ? 'rgba(56, 189, 248, 0.2)' : 'rgba(255, 255, 255, 0.04)',
                border: `1px solid ${selectedKindFilter === k ? 'var(--accent-cyan)' : 'rgba(255, 255, 255, 0.08)'}`,
                color: selectedKindFilter === k ? 'var(--accent-cyan)' : 'var(--text-muted)',
                borderRadius: 4,
                padding: '2px 7px',
                fontSize: '0.68rem',
                cursor: 'pointer',
                textTransform: 'capitalize',
              }}
            >
              {k}
            </button>
          ))}
        </div>

        {/* Clusters Accordion List */}
        <div style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 8, paddingRight: 4 }}>
          {clusters
            .filter((c) => {
              if (!clusterSearchTerm) return true;
              const term = clusterSearchTerm.toLowerCase();
              return c.name.toLowerCase().includes(term) || c.nodes.some((n) => n.label.toLowerCase().includes(term));
            })
            .map((c) => {
              const color = GALAXY_COLORS[Math.abs(c.id) % GALAXY_COLORS.length];
              const isExpanded = selectedClusterId === c.id;
              const isIsolated = isolatedClusterId === c.id;

              const filteredNodes = c.nodes.filter((node) => {
                const matchesTerm = !clusterSearchTerm || node.label.toLowerCase().includes(clusterSearchTerm.toLowerCase());
                const matchesKind = selectedKindFilter === 'all' || node.kind.toLowerCase() === selectedKindFilter.toLowerCase();
                return matchesTerm && matchesKind;
              });

              return (
                <div
                  key={c.id}
                  style={{
                    borderRadius: 8,
                    background: isIsolated
                      ? 'rgba(168, 85, 247, 0.15)'
                      : isExpanded
                      ? 'rgba(56, 189, 248, 0.08)'
                      : 'rgba(255, 255, 255, 0.03)',
                    border: `1px solid ${
                      isIsolated
                        ? 'rgba(168, 85, 247, 0.6)'
                        : isExpanded
                        ? 'rgba(56, 189, 248, 0.4)'
                        : 'rgba(255, 255, 255, 0.06)'
                    }`,
                    overflow: 'hidden',
                    transition: 'all 0.18s ease',
                  }}
                >
                  <div
                    onClick={() => handleFocusCluster(c.id)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '8px 10px',
                      cursor: 'pointer',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, overflow: 'hidden' }}>
                      <div
                        style={{
                          width: 9,
                          height: 9,
                          borderRadius: '50%',
                          backgroundColor: color,
                          boxShadow: `0 0 8px ${color}`,
                          flexShrink: 0,
                        }}
                      />
                      <span
                        style={{
                          fontSize: '0.8rem',
                          fontWeight: 600,
                          whiteSpace: 'nowrap',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          color: isIsolated ? 'var(--accent-purple)' : isExpanded ? 'var(--accent-cyan)' : '#e2e8f0',
                        }}
                      >
                        {c.name}
                      </span>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <span
                        style={{
                          fontSize: '0.7rem',
                          color: 'var(--text-muted)',
                          background: 'rgba(255, 255, 255, 0.06)',
                          borderRadius: 4,
                          padding: '2px 6px',
                        }}
                      >
                        {c.node_count}
                      </span>
                      {isExpanded ? <ChevronDown size={14} color="var(--text-muted)" /> : <ChevronRight size={14} color="var(--text-muted)" />}
                    </div>
                  </div>

                  {/* Expanded Cluster Actions & Symbols */}
                  {isExpanded && (
                    <div
                      style={{
                        padding: '6px 10px 10px 10px',
                        borderTop: '1px solid rgba(255, 255, 255, 0.05)',
                        display: 'flex',
                        flexDirection: 'column',
                        gap: 6,
                      }}
                    >
                      {/* Cluster Action Pills */}
                      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
                        <button
                          onClick={(e) => handleToggleIsolateGalaxy(c.id, e)}
                          className={isIsolated ? 'cyber-button-purple' : 'cyber-button-secondary'}
                          style={{ padding: '3px 8px', fontSize: '0.7rem' }}
                          title="Isolate galaxy in 3D WebGL view"
                        >
                          <Maximize2 size={11} />
                          <span>{isIsolated ? 'Exit Isolate' : 'Isolate Galaxy'}</span>
                        </button>

                        <button
                          onClick={(e) => handleCopyClusterArchitecture(c, e)}
                          className="cyber-button-secondary"
                          style={{ padding: '3px 8px', fontSize: '0.7rem' }}
                          title="Copy subsystem symbols for AI agent"
                        >
                          <Copy size={11} />
                          <span>Copy Architecture</span>
                        </button>
                      </div>

                      {/* Filtered symbols list */}
                      <div
                        style={{
                          display: 'flex',
                          flexDirection: 'column',
                          gap: 3,
                          maxHeight: 200,
                          overflowY: 'auto',
                          marginTop: 4,
                        }}
                      >
                        {filteredNodes.map((node) => {
                          const kindClass =
                            node.kind === 'function'
                              ? 'kind-badge-function'
                              : node.kind === 'struct' || node.kind === 'class'
                              ? 'kind-badge-struct'
                              : node.kind === 'import'
                              ? 'kind-badge-import'
                              : 'kind-badge-default';

                          return (
                            <div
                              key={node.id}
                              onClick={() => handleFocusNode(node)}
                              style={{
                                display: 'flex',
                                alignItems: 'center',
                                justifyContent: 'space-between',
                                padding: '4px 6px',
                                borderRadius: 4,
                                cursor: 'pointer',
                                background: selectedNode?.id === node.id ? 'rgba(56, 189, 248, 0.2)' : 'transparent',
                                fontSize: '0.75rem',
                                color: selectedNode?.id === node.id ? 'var(--accent-cyan)' : 'var(--text-secondary)',
                              }}
                            >
                              <span style={{ fontFamily: 'var(--code-font)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                                {node.label}
                              </span>
                              <span className={`kind-badge ${kindClass}`}>{node.kind}</span>
                            </div>
                          );
                        })}

                        {filteredNodes.length === 0 && (
                          <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textAlign: 'center', padding: 8 }}>
                            No symbols match current filter.
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              );
            })}

          {clusters.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.8rem', padding: 20 }}>
              No galaxy clusters computed yet. Click <strong>"Compute Galaxy Clusters"</strong> above to partition the AST graph!
            </div>
          )}
        </div>
      </aside>

      {/* ─── 4. RIGHT COLLAPSIBLE DRAWER: CONTEXT CONDENSER & NODE INSPECTOR ── */}
      {selectedNode && (
        <aside
          className="glass-panel drawer-right"
          style={{
            position: 'absolute',
            top: 86,
            right: 14,
            bottom: 14,
            width: 370,
            zIndex: 90,
            display: 'flex',
            flexDirection: 'column',
            padding: 18,
            gap: 14,
            overflowY: 'auto',
          }}
        >
          {/* Header */}
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
              <span
                style={{
                  background: 'rgba(56, 189, 248, 0.15)',
                  color: 'var(--accent-cyan)',
                  borderRadius: 6,
                  padding: '3px 8px',
                  fontSize: '0.7rem',
                  fontWeight: 700,
                  textTransform: 'uppercase',
                }}
              >
                {selectedNode.kind}
              </span>
              <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                {selectedNode.language}
              </span>
            </div>
            <button
              onClick={() => setSelectedNode(null)}
              style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
            >
              <X size={16} />
            </button>
          </div>

          <div>
            <h3
              style={{
                margin: 0,
                fontSize: '1.15rem',
                fontWeight: 700,
                color: '#fff',
                fontFamily: 'var(--code-font)',
                wordBreak: 'break-all',
              }}
            >
              {selectedNode.label}
            </h3>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginTop: 4, wordBreak: 'break-all' }}>
              📁 {selectedNode.file_path}
              {selectedNode.line_start ? ` : L${selectedNode.line_start}` : ''}
            </div>
          </div>

          {/* Subsystem Community Badge */}
          {selectedNode.community !== undefined && selectedNode.community !== null && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 8,
                padding: '6px 10px',
                background: 'rgba(255, 255, 255, 0.04)',
                borderRadius: 6,
                border: '1px solid rgba(255, 255, 255, 0.08)',
              }}
            >
              <div
                style={{
                  width: 8,
                  height: 8,
                  borderRadius: '50%',
                  backgroundColor: GALAXY_COLORS[Math.abs(selectedNode.community) % GALAXY_COLORS.length],
                }}
              />
              <span style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>
                Galaxy Cluster #{selectedNode.community}
              </span>
            </div>
          )}

          {/* Code Snippet Box */}
          {selectedNode.text && (
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', fontWeight: 600 }}>SOURCE SNIPPET</div>
              <pre
                className="code-font"
                style={{
                  margin: 0,
                  padding: 12,
                  background: 'rgba(0, 0, 0, 0.55)',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  borderRadius: 8,
                  fontSize: '0.75rem',
                  lineHeight: 1.45,
                  overflowX: 'auto',
                  color: '#e2e8f0',
                  maxHeight: 200,
                }}
              >
                <code>{selectedNode.text}</code>
              </pre>
            </div>
          )}

          {/* Agent Context Condenser Button (R-028) */}
          <div style={{ marginTop: 'auto', paddingTop: 12, borderTop: '1px solid rgba(255, 255, 255, 0.08)' }}>
            <button
              onClick={() => handleFetchCondensed(selectedNode.label)}
              className="cyber-button"
              style={{ width: '100%', justifyContent: 'center' }}
            >
              {copied ? <Check size={15} color="#34d399" /> : <Copy size={15} />}
              <span>{copied ? 'Copied Subgraph to Clipboard!' : 'Copy Prompt Slice (<1500 tokens)'}</span>
            </button>
            <div style={{ fontSize: '0.68rem', color: 'var(--text-muted)', textAlign: 'center', marginTop: 6 }}>
              Condenses 2-hop AST call trace ready to paste into LLM context.
            </div>

            {condensedText && (
              <details style={{ marginTop: 8, fontSize: '0.72rem', color: 'var(--text-muted)' }}>
                <summary style={{ cursor: 'pointer' }}>Preview Condensed Markdown</summary>
                <pre
                  className="code-font"
                  style={{
                    margin: '6px 0 0',
                    padding: 8,
                    background: 'rgba(0, 0, 0, 0.5)',
                    borderRadius: 6,
                    maxHeight: 140,
                    overflowY: 'auto',
                    whiteSpace: 'pre-wrap',
                  }}
                >
                  {condensedText}
                </pre>
              </details>
            )}
          </div>
        </aside>
      )}

      {/* ─── 5. FUTURISTIC DIRECTORY BROWSER MODAL (INGEST DIALOG) ─────────── */}
      {isBrowserModalOpen && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            zIndex: 200,
            background: 'rgba(0, 0, 0, 0.75)',
            backdropFilter: 'blur(16px)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 20,
          }}
          onClick={(e) => {
            if (e.target === e.currentTarget && !isIngesting) setIsBrowserModalOpen(false);
          }}
        >
          <div
            className="glass-panel"
            style={{
              width: '100%',
              maxWidth: 680,
              maxHeight: '85vh',
              display: 'flex',
              flexDirection: 'column',
              padding: 24,
              gap: 16,
              boxShadow: '0 25px 60px -15px rgba(0, 0, 0, 0.9)',
              border: '1px solid rgba(56, 189, 248, 0.3)',
            }}
          >
            {/* Modal Header */}
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <div
                  style={{
                    width: 36,
                    height: 36,
                    borderRadius: 8,
                    background: 'linear-gradient(135deg, #0284c7, #6366f1)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <FolderInput size={20} color="#fff" />
                </div>
                <div>
                  <h3 style={{ margin: 0, fontSize: '1.05rem', fontWeight: 700, color: '#f8fafc' }}>
                    Select Codebase to Ingest
                  </h3>
                  <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                    Traverse folders dynamically and index any repository into Omni-Graph.
                  </div>
                </div>
              </div>
              <button
                onClick={() => !isIngesting && setIsBrowserModalOpen(false)}
                style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
              >
                <X size={18} />
              </button>
            </div>

            {/* Quick Bookmark Jumps */}
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
              <span style={{ fontSize: '0.7rem', color: 'var(--text-muted)', fontWeight: 600 }}>QUICK JUMP:</span>
              <button
                onClick={() => fetchDirectory('/workspace')}
                className="cyber-button-secondary"
                style={{ padding: '3px 10px', fontSize: '0.72rem' }}
              >
                💻 Monorepo (/workspace)
              </button>
              <button
                onClick={() => fetchDirectory('/Users')}
                className="cyber-button-secondary"
                style={{ padding: '3px 10px', fontSize: '0.72rem' }}
              >
                👤 Host Users (/Users)
              </button>
              <button
                onClick={() => fetchDirectory('/workspace/tools')}
                className="cyber-button-secondary"
                style={{ padding: '3px 10px', fontSize: '0.72rem' }}
              >
                📦 Tools Monorepo
              </button>
            </div>

            {/* Breadcrumb Path Bar */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                padding: '8px 12px',
                background: 'rgba(0, 0, 0, 0.4)',
                borderRadius: 8,
                border: '1px solid rgba(255, 255, 255, 0.08)',
                fontSize: '0.8rem',
                overflowX: 'auto',
              }}
            >
              {parentPath && (
                <button
                  onClick={() => fetchDirectory(parentPath)}
                  title="Up one level"
                  style={{
                    background: 'rgba(255, 255, 255, 0.08)',
                    border: 'none',
                    borderRadius: 4,
                    padding: '3px 6px',
                    color: 'var(--accent-cyan)',
                    cursor: 'pointer',
                    display: 'flex',
                    alignItems: 'center',
                    marginRight: 4,
                  }}
                >
                  <ArrowUp size={14} />
                </button>
              )}

              <span
                onClick={() => fetchDirectory('/')}
                style={{ color: 'var(--accent-cyan)', cursor: 'pointer', fontWeight: 600 }}
              >
                root
              </span>
              {browsePath
                .split('/')
                .filter(Boolean)
                .map((segment, idx, arr) => {
                  const subPath = '/' + arr.slice(0, idx + 1).join('/');
                  return (
                    <React.Fragment key={subPath}>
                      <span style={{ color: 'var(--text-muted)' }}>/</span>
                      <span
                        onClick={() => fetchDirectory(subPath)}
                        style={{
                          color: idx === arr.length - 1 ? '#fff' : 'var(--text-secondary)',
                          fontWeight: idx === arr.length - 1 ? 700 : 400,
                          cursor: 'pointer',
                        }}
                      >
                        {segment}
                      </span>
                    </React.Fragment>
                  );
                })}
            </div>

            {/* Directory Entries List */}
            <div
              style={{
                flex: 1,
                minHeight: 250,
                maxHeight: 330,
                overflowY: 'auto',
                background: 'rgba(0, 0, 0, 0.25)',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                borderRadius: 8,
                padding: 6,
                display: 'flex',
                flexDirection: 'column',
                gap: 4,
              }}
            >
              {isLoadingDir ? (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', gap: 8, color: 'var(--text-muted)' }}>
                  <RefreshCw size={18} className="pulsing-dot" color="var(--accent-cyan)" />
                  <span>Loading directory contents...</span>
                </div>
              ) : dirEntries.length > 0 ? (
                dirEntries.map((entry) => (
                  <div
                    key={entry.path}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '8px 12px',
                      borderRadius: 6,
                      background: selectedFolderForIngest === entry.path ? 'rgba(56, 189, 248, 0.12)' : 'rgba(255, 255, 255, 0.02)',
                      border: `1px solid ${
                        selectedFolderForIngest === entry.path
                          ? 'rgba(56, 189, 248, 0.5)'
                          : entry.is_codebase
                          ? 'rgba(168, 85, 247, 0.35)'
                          : 'transparent'
                      }`,
                      cursor: 'pointer',
                      transition: 'all 0.15s ease',
                    }}
                    onClick={() => {
                      setSelectedFolderForIngest(entry.path);
                      setCustomProjectName(entry.name);
                    }}
                    onDoubleClick={() => fetchDirectory(entry.path)}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 10, overflow: 'hidden' }}>
                      <Folder
                        size={17}
                        color={entry.is_codebase ? 'var(--accent-purple)' : 'var(--accent-cyan)'}
                        style={{ flexShrink: 0 }}
                      />
                      <div>
                        <div style={{ fontWeight: 600, fontSize: '0.82rem', color: '#f8fafc' }}>
                          {entry.name}
                        </div>
                        {entry.is_codebase && (
                          <div style={{ fontSize: '0.68rem', color: 'var(--accent-purple)', display: 'flex', alignItems: 'center', gap: 4 }}>
                            <Code size={11} />
                            <span>Detected Codebase {entry.languages.length > 0 ? `(${entry.languages.join(', ')})` : ''}</span>
                          </div>
                        )}
                      </div>
                    </div>

                    <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          fetchDirectory(entry.path);
                        }}
                        className="cyber-button-secondary"
                        style={{ padding: '3px 8px', fontSize: '0.7rem' }}
                        title="Enter folder"
                      >
                        <span>Open</span>
                        <ChevronRight size={12} />
                      </button>
                    </div>
                  </div>
                ))
              ) : (
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-muted)', fontSize: '0.8rem' }}>
                  No subdirectories in this path.
                </div>
              )}
            </div>

            {/* Ingestion Settings & Status Footer */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <div style={{ flex: 1 }}>
                  <label style={{ fontSize: '0.7rem', color: 'var(--text-muted)', fontWeight: 600 }}>TARGET PATH</label>
                  <input
                    type="text"
                    value={selectedFolderForIngest}
                    onChange={(e) => setSelectedFolderForIngest(e.target.value)}
                    style={{
                      width: '100%',
                      background: 'rgba(0, 0, 0, 0.4)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '6px 10px',
                      color: '#e2e8f0',
                      fontSize: '0.78rem',
                      marginTop: 2,
                    }}
                  />
                </div>
                <div style={{ width: 180 }}>
                  <label style={{ fontSize: '0.7rem', color: 'var(--text-muted)', fontWeight: 600 }}>WORKSPACE NAME</label>
                  <input
                    type="text"
                    placeholder="e.g. session-explorer"
                    value={customProjectName}
                    onChange={(e) => setCustomProjectName(e.target.value)}
                    style={{
                      width: '100%',
                      background: 'rgba(0, 0, 0, 0.4)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 6,
                      padding: '6px 10px',
                      color: '#e2e8f0',
                      fontSize: '0.78rem',
                      marginTop: 2,
                    }}
                  />
                </div>
              </div>

              {/* Auto-Cluster Checkbox */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '2px 0' }}>
                <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer', fontSize: '0.76rem', color: '#e2e8f0' }}>
                  <input
                    type="checkbox"
                    checked={autoClusterAfterIngest}
                    onChange={(e) => setAutoClusterAfterIngest(e.target.checked)}
                    style={{ accentColor: 'var(--accent-purple)', cursor: 'pointer', width: 15, height: 15 }}
                  />
                  <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                    <Sparkles size={13} color="var(--accent-purple)" />
                    <span>Auto-compute Galaxy Clusters immediately after ingestion (Recommended)</span>
                  </span>
                </label>
              </div>

              {ingestNotice && (
                <div
                  style={{
                    padding: '8px 12px',
                    borderRadius: 6,
                    background: ingestNotice.startsWith('✅') || ingestNotice.startsWith('✨') ? 'rgba(52, 211, 153, 0.1)' : 'rgba(56, 189, 248, 0.1)',
                    border: `1px solid ${ingestNotice.startsWith('✅') || ingestNotice.startsWith('✨') ? 'rgba(52, 211, 153, 0.3)' : 'rgba(56, 189, 248, 0.3)'}`,
                    fontSize: '0.75rem',
                    color: ingestNotice.startsWith('✅') || ingestNotice.startsWith('✨') ? '#34d399' : '#38bdf8',
                  }}
                >
                  {ingestNotice}
                </div>
              )}

              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 10, marginTop: 4 }}>
                <button
                  onClick={() => setIsBrowserModalOpen(false)}
                  disabled={isIngesting}
                  className="cyber-button-secondary"
                >
                  Cancel
                </button>
                <button
                  onClick={() => handleIngestExecution(selectedFolderForIngest, customProjectName)}
                  disabled={isIngesting || !selectedFolderForIngest}
                  className="cyber-button"
                >
                  {isIngesting ? <RefreshCw size={15} className="pulsing-dot" /> : <Sparkles size={15} />}
                  <span>{isIngesting ? 'Ingesting Codebase...' : 'Start Ingestion'}</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
