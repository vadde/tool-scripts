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
  BarChart3,
  Radio,
  Activity,
  Play,
  Square,
  Boxes,
  ArrowLeft,
} from 'lucide-react';
import AgentAnalytics from './AgentAnalytics';
import BoundaryContractCard from './BoundaryContractCard';
import SubsystemTopologyHub, { GalaxyRecord, GalaxyDependencyEdge } from './SubsystemTopologyHub';

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
  root_path?: string;
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

export interface WatcherStatus {
  workspace: string;
  path: string;
  status: string;
  started_at: string;
  files_tracked: number;
  last_sync: string | null;
  events_processed: number;
  files_reindexed: number;
  files_deleted: number;
  avg_sync_ms: number;
  cluster_status: string;
  last_cluster_at: string | null;
  debounce_ms: number;
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
  // Navigation View Mode: 3D Graph Studio vs Agent Analytics & LSP Telemetry
  const [viewMode, setViewMode] = useState<'graph' | 'analytics'>('graph');

  // Graph state
  const [nodes, setNodes] = useState<GraphNode[]>([]);
  const [links, setLinks] = useState<GraphLink[]>([]);
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [nodeHistory, setNodeHistory] = useState<GraphNode[]>([]);
  const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);

  // Workspaces state
  const [workspaces, setWorkspaces] = useState<WorkspaceInfo[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState<string | null>(null);
  const selectedWorkspaceRef = useRef<string | null>(selectedWorkspace);
  const [isWorkspaceDropdownOpen, setIsWorkspaceDropdownOpen] = useState(false);

  useEffect(() => {
    selectedWorkspaceRef.current = selectedWorkspace;
  }, [selectedWorkspace]);

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
  const [visibleClusterCount, setVisibleClusterCount] = useState(60);
  const [expandedAllNodes, setExpandedAllNodes] = useState<{ [clusterId: number]: boolean }>({});

  // Dynamic Macro Topology state (R-029, R-030)
  const [topologyGalaxies, setTopologyGalaxies] = useState<GalaxyRecord[]>([]);
  const [topologyDependencies, setTopologyDependencies] = useState<GalaxyDependencyEdge[]>([]);
  const [galaxyViewMode, setGalaxyViewMode] = useState<'topology' | 'flat'>('topology');

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

  // Live Watch state
  const [activeWatchers, setActiveWatchers] = useState<WatcherStatus[]>([]);
  const [isWatchModalOpen, setIsWatchModalOpen] = useState(false);
  const [isWatchBarMinimized, setIsWatchBarMinimized] = useState(false);
  const [watchToast, setWatchToast] = useState<string | null>(null);
  const [watchTargetFolder, setWatchTargetFolder] = useState<string>('/workspace');
  const [watchCustomProject, setWatchCustomProject] = useState<string>('');
  const [watchDebounceMs, setWatchDebounceMs] = useState<number>(500);
  const [isStartingWatch, setIsStartingWatch] = useState<boolean>(false);

  const cosmographRef = useRef<any>(null);
  const workspaceDropdownRef = useRef<HTMLDivElement>(null);

  // ─── 1. Data Fetching ──────────────────────────────────────────────────────
  const fetchWatchers = async () => {
    try {
      const res = await fetch('/api/watch/status');
      if (res.ok) {
        const list: WatcherStatus[] = await res.json();
        setActiveWatchers(list || []);
      }
    } catch (err) {
      console.warn('Failed to load watchers status', err);
    }
  };

  const handleStartWatch = async (path: string, project?: string, debounceMs: number = 500) => {
    setIsStartingWatch(true);
    try {
      const res = await fetch('/api/watch/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          path,
          project: project || undefined,
          debounce_ms: debounceMs,
        }),
      });
      if (res.ok) {
        const data = await res.json();
        setWatchToast(`🟢 Watching ${data.workspace} (${data.files_tracked} files)`);
        setTimeout(() => setWatchToast(null), 4000);
        await fetchWatchers();
        // Modal stays open so user can watch multiple or inspect telemetry until clicking close
      } else {
        const errJson = await res.json();
        alert(`Failed to start watch: ${errJson.error || res.statusText}`);
      }
    } catch (err: any) {
      alert(`Error starting watch: ${err.message}`);
    } finally {
      setIsStartingWatch(false);
    }
  };

  const handleStopWatch = async (workspace: string) => {
    try {
      const res = await fetch('/api/watch/stop', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workspace }),
      });
      if (res.ok) {
        setWatchToast(`🛑 Stopped watching ${workspace}`);
        setTimeout(() => setWatchToast(null), 3500);
        await fetchWatchers();
      }
    } catch (err) {
      console.warn('Failed to stop watch', err);
    }
  };

  const loadWorkspaces = async () => {
    try {
      const res = await fetch('/api/workspaces');
      if (res.ok) {
        const data: WorkspaceInfo[] = await res.json();
        const seen = new Set<string>();
        const uniqueWorkspaces: WorkspaceInfo[] = [];
        (data || []).forEach(ws => {
          const name = ws.workspace?.trim();
          if (!name || name === 'workspace' || name === 'default' || name === 'global') return;
          const key = name.toLowerCase();
          if (!seen.has(key)) {
            seen.add(key);
            uniqueWorkspaces.push(ws);
          }
        });
        setWorkspaces(uniqueWorkspaces);
      }
    } catch (err) {
      console.warn('Failed to load workspaces', err);
    }
  };

  const loadGalaxyTopology = async (wsFilter?: string | null) => {
    try {
      const ws = wsFilter !== undefined ? wsFilter : selectedWorkspaceRef.current;
      const qs = ws ? `workspace=${encodeURIComponent(ws)}&_t=${Date.now()}` : `_t=${Date.now()}`;
      const res = await fetch(`/api/galaxy/topology?${qs}`);
      if (res.ok) {
        const json = await res.json();
        setTopologyGalaxies(json.galaxies || []);
        setTopologyDependencies(json.dependencies || []);
      }
    } catch (err) {
      console.warn('Failed to load galaxy topology', err);
    }
  };

  const loadGraph = async (wsFilter?: string | null) => {
    try {
      const qs = wsFilter
        ? `workspace=${encodeURIComponent(wsFilter)}&_t=${Date.now()}`
        : `_t=${Date.now()}`;
      const url = `/api/graph?${qs}`;
      const res = await fetch(url);
      if (res.ok) {
        const gJson = await res.json();
        setNodes(gJson.nodes || []);
        setLinks(gJson.links || []);
      }
      await loadGalaxyTopology(wsFilter);
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
      } else {
        setHealth(null);
      }
    } catch (err) {
      console.warn('Failed to load health status', err);
      setHealth(null);
    }
  };

  useEffect(() => {
    loadHealth();
    loadWorkspaces();
    loadGraph(selectedWorkspace);
    loadGalaxyTopology(selectedWorkspace);
    fetchWatchers();

    const interval = setInterval(() => {
      loadHealth();
      fetchWatchers();
    }, 4000);

    // Real-time SSE Delta stream connection
    let es: EventSource | null = null;
    try {
      es = new EventSource('/api/watch/events');
      es.addEventListener('delta', (e: MessageEvent) => {
        try {
          const evt = JSON.parse(e.data);
          setWatchToast(`⚡ Live Delta: [${evt.workspace}] ${evt.kind} ${evt.file_path} (${evt.sync_ms || 0}ms)`);
          setTimeout(() => setWatchToast(null), 3500);
          fetchWatchers();
          loadWorkspaces();
        } catch (err) {
          console.warn('SSE parse error', err);
        }
      });

      es.addEventListener('cluster', (e: MessageEvent) => {
        try {
          const evt = JSON.parse(e.data);
          setWatchToast(`🌌 Dynamic Live Modularity: [${evt.workspace}] ${evt.total_clusters} galaxies re-indexed`);
          setTimeout(() => setWatchToast(null), 3500);
          loadGalaxyTopology(selectedWorkspaceRef.current);
          loadGraph(selectedWorkspaceRef.current);
        } catch (err) {
          console.warn('SSE cluster parse error', err);
        }
      });
    } catch (err) {
      console.warn('SSE connection error', err);
    }

    return () => {
      clearInterval(interval);
      if (es) es.close();
    };
  }, []);

  // Dismiss workspace dropdown on outside click or Escape key
  useEffect(() => {
    if (!isWorkspaceDropdownOpen) return;

    const handleClickOutside = (event: MouseEvent | TouchEvent) => {
      if (
        workspaceDropdownRef.current &&
        !workspaceDropdownRef.current.contains(event.target as Node)
      ) {
        setIsWorkspaceDropdownOpen(false);
      }
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsWorkspaceDropdownOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside, true);
    document.addEventListener('touchstart', handleClickOutside, true);
    document.addEventListener('keydown', handleKeyDown);

    return () => {
      document.removeEventListener('mousedown', handleClickOutside, true);
      document.removeEventListener('touchstart', handleClickOutside, true);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [isWorkspaceDropdownOpen]);

  const handleSelectWorkspace = (ws: string | null) => {
    setSelectedWorkspace(ws);
    setIsWorkspaceDropdownOpen(false);
    setSelectedNode(null);
    setNodeHistory([]);
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

    if (map.size > 0) {
      const list = Array.from(map.entries()).map(([cid, clusterNodes]) => {
        const paths = clusterNodes.map((n) => {
          const parts = n.file_path.split('/');
          return parts.length > 1 ? parts.slice(0, -1).join('/') : n.file_path;
        });
        const counts: Record<string, number> = {};
        paths.forEach((p) => (counts[p] = (counts[p] || 0) + 1));
        const dominant = Object.entries(counts).sort((a, b) => b[1] - a[1])[0]?.[0] || 'subsystem';

        const clusterTitle =
          clusterNodes.length === 1
            ? `Cluster #${cid}: ${dominant} • ${clusterNodes[0].label}`
            : `Cluster #${cid}: ${dominant}`;

        return {
          id: cid,
          name: clusterTitle,
          node_count: clusterNodes.length,
          nodes: clusterNodes,
          isCommunity: true,
        };
      });

      return list.sort((a, b) => b.node_count - a.node_count);
    }

    // Fallback: Group by directory if communities are not yet computed
    const dirMap = new Map<string, GraphNode[]>();
    for (const node of nodes) {
      const parts = node.file_path.split('/');
      const dir = parts.length > 1 ? parts.slice(0, -1).join('/') : 'root';
      if (!dirMap.has(dir)) {
        dirMap.set(dir, []);
      }
      dirMap.get(dir)!.push(node);
    }

    let syntheticId = 1;
    const dirList = Array.from(dirMap.entries()).map(([dir, dirNodes]) => ({
      id: syntheticId++,
      name: `Subsystem: ${dir}`,
      node_count: dirNodes.length,
      nodes: dirNodes,
      isCommunity: false,
    }));

    return dirList.sort((a, b) => b.node_count - a.node_count);
  }, [nodes]);

  const filteredClusters = useMemo(() => {
    if (!clusterSearchTerm.trim()) return clusters;
    const term = clusterSearchTerm.toLowerCase();
    return clusters.filter(
      (c) => c.name.toLowerCase().includes(term) || c.nodes.some((n) => n.label.toLowerCase().includes(term))
    );
  }, [clusters, clusterSearchTerm]);

  useEffect(() => {
    setVisibleClusterCount(60);
  }, [selectedWorkspace, clusterSearchTerm]);

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
      setClusterToast(
        '❌ Connection error: Backend stack may be offline. Run "make omni-graph" in your terminal to start services.'
      );
      setTimeout(() => setClusterToast(null), 6000);
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
      setIngestNotice('❌ Connection error: Backend stack may be offline. Run "make omni-graph" in your terminal to start.');
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
    setNodeHistory([]);
    if (cosmographRef.current) {
      cosmographRef.current.selectNodes([node]);
    }
  };

  const getNodeColor = (node: GraphNode) => {
    if (node.community !== undefined && node.community !== null) {
      return GALAXY_COLORS[Math.abs(node.community) % GALAXY_COLORS.length];
    }
    // Color-code by directory subsystem so graph clusters visually even before LPA clustering
    const parts = node.file_path.split('/');
    const dir = parts.length > 1 ? parts.slice(0, -1).join('/') : 'root';
    let hash = 0;
    for (let i = 0; i < dir.length; i++) {
      hash = (hash << 5) - hash + dir.charCodeAt(i);
      hash |= 0;
    }
    return GALAXY_COLORS[Math.abs(hash) % GALAXY_COLORS.length];
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
          <div ref={workspaceDropdownRef} style={{ position: 'relative', marginLeft: 8 }}>
            <button
              type="button"
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
              <ChevronDown
                size={14}
                color="var(--text-muted)"
                style={{
                  transform: isWorkspaceDropdownOpen ? 'rotate(180deg)' : 'none',
                  transition: 'transform 0.2s ease',
                }}
              />
            </button>

            {isWorkspaceDropdownOpen && (
              <div
                className="glass-panel"
                style={{
                  position: 'absolute',
                  top: 'calc(100% + 8px)',
                  left: 0,
                  width: 290,
                  maxHeight: '70vh',
                  overflowY: 'auto',
                  zIndex: 110,
                  padding: 8,
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 4,
                  border: '1px solid rgba(56, 189, 248, 0.3)',
                  boxShadow: '0 20px 40px rgba(0, 0, 0, 0.85), 0 0 20px rgba(56, 189, 248, 0.12)',
                }}
              >
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', padding: '4px 8px', fontWeight: 600 }}>
                  SELECT ACTIVE CODEBASE
                </div>
                <button
                  type="button"
                  onClick={() => handleSelectWorkspace(null)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '8px 10px',
                    borderRadius: 6,
                    background: selectedWorkspace === null ? 'rgba(56, 189, 248, 0.18)' : 'transparent',
                    border: 'none',
                    color: selectedWorkspace === null ? 'var(--accent-cyan)' : 'var(--text-primary)',
                    cursor: 'pointer',
                    fontSize: '0.8rem',
                    textAlign: 'left',
                    transition: 'all 0.15s ease',
                  }}
                  onMouseEnter={(e) => {
                    if (selectedWorkspace !== null) e.currentTarget.style.background = 'rgba(255, 255, 255, 0.06)';
                  }}
                  onMouseLeave={(e) => {
                    if (selectedWorkspace !== null) e.currentTarget.style.background = 'transparent';
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
                    type="button"
                    onClick={() => handleSelectWorkspace(ws.workspace)}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      justifyContent: 'space-between',
                      padding: '8px 10px',
                      borderRadius: 6,
                      background: selectedWorkspace === ws.workspace ? 'rgba(56, 189, 248, 0.18)' : 'transparent',
                      border: 'none',
                      color: selectedWorkspace === ws.workspace ? 'var(--accent-cyan)' : 'var(--text-primary)',
                      cursor: 'pointer',
                      fontSize: '0.8rem',
                      textAlign: 'left',
                      transition: 'all 0.15s ease',
                    }}
                    onMouseEnter={(e) => {
                      if (selectedWorkspace !== ws.workspace) e.currentTarget.style.background = 'rgba(255, 255, 255, 0.06)';
                    }}
                    onMouseLeave={(e) => {
                      if (selectedWorkspace !== ws.workspace) e.currentTarget.style.background = 'transparent';
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

        {/* View Mode Switcher: 3D Graph Studio vs Agent Analytics & LSP Telemetry */}
        <div
          style={{
            display: 'flex',
            background: 'rgba(0, 0, 0, 0.45)',
            padding: 3,
            borderRadius: 8,
            border: '1px solid rgba(255, 255, 255, 0.1)',
          }}
        >
          <button
            type="button"
            onClick={() => setViewMode('graph')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '5px 12px',
              borderRadius: 6,
              border: 'none',
              fontSize: '0.76rem',
              fontWeight: viewMode === 'graph' ? 600 : 400,
              cursor: 'pointer',
              background:
                viewMode === 'graph'
                  ? 'linear-gradient(135deg, rgba(2, 132, 199, 0.4), rgba(147, 51, 234, 0.4))'
                  : 'transparent',
              color: viewMode === 'graph' ? '#fff' : 'var(--text-secondary)',
              boxShadow: viewMode === 'graph' ? '0 0 10px rgba(56, 189, 248, 0.3)' : 'none',
            }}
          >
            <Compass size={14} color={viewMode === 'graph' ? 'var(--accent-cyan)' : 'var(--text-muted)'} />
            <span>3D Graph Studio</span>
          </button>

          <button
            type="button"
            onClick={() => setViewMode('analytics')}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 6,
              padding: '5px 12px',
              borderRadius: 6,
              border: 'none',
              fontSize: '0.76rem',
              fontWeight: viewMode === 'analytics' ? 600 : 400,
              cursor: 'pointer',
              background:
                viewMode === 'analytics'
                  ? 'linear-gradient(135deg, rgba(56, 189, 248, 0.3), rgba(52, 211, 153, 0.3))'
                  : 'transparent',
              color: viewMode === 'analytics' ? '#38bdf8' : 'var(--text-secondary)',
              boxShadow: viewMode === 'analytics' ? '0 0 10px rgba(56, 189, 248, 0.3)' : 'none',
            }}
          >
            <BarChart3 size={14} color={viewMode === 'analytics' ? '#38bdf8' : 'var(--text-muted)'} />
            <span>Agent Analytics & LSP</span>
            <span
              style={{
                background: 'rgba(52, 211, 153, 0.25)',
                color: '#34d399',
                borderRadius: 10,
                padding: '1px 5px',
                fontSize: '0.62rem',
                fontWeight: 700,
              }}
            >
              Omni-DB
            </span>
          </button>
        </div>

        {/* Live Watch Status Button in Header */}
        <button
          type="button"
          onClick={() => setIsWatchModalOpen(true)}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 7,
            padding: '5px 12px',
            borderRadius: 8,
            border: activeWatchers.length > 0 
              ? '1px solid rgba(239, 68, 68, 0.45)' 
              : '1px solid rgba(255, 255, 255, 0.1)',
            background: activeWatchers.length > 0 
              ? 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(244, 63, 94, 0.15))' 
              : 'rgba(0, 0, 0, 0.45)',
            color: activeWatchers.length > 0 ? '#fca5a5' : 'var(--text-secondary)',
            fontSize: '0.76rem',
            fontWeight: 600,
            cursor: 'pointer',
            transition: 'all 0.2s ease',
            boxShadow: activeWatchers.length > 0 ? '0 0 12px rgba(239, 68, 68, 0.25)' : 'none',
          }}
          title="Manage Live Watchers: auto-sync AST and vectors as code changes"
        >
          {activeWatchers.length > 0 ? (
            <span
              style={{
                width: 8,
                height: 8,
                borderRadius: '50%',
                background: '#ef4444',
                boxShadow: '0 0 8px #ef4444',
                display: 'inline-block',
              }}
              className="pulsing-dot"
            />
          ) : (
            <Radio size={14} color="var(--text-muted)" />
          )}
          <span>
            {activeWatchers.length > 0 ? `LIVE WATCH (${activeWatchers.length} Active)` : 'Live Watch'}
          </span>
          {activeWatchers.length > 0 ? (
            <span
              style={{
                background: 'rgba(239, 68, 68, 0.3)',
                color: '#f87171',
                borderRadius: 10,
                padding: '1px 6px',
                fontSize: '0.62rem',
                fontWeight: 700,
              }}
            >
              RECORDING
            </span>
          ) : (
            <span
              style={{
                background: 'rgba(255, 255, 255, 0.08)',
                color: 'var(--text-muted)',
                borderRadius: 10,
                padding: '1px 6px',
                fontSize: '0.62rem',
              }}
            >
              Idle
            </span>
          )}
        </button>

        {/* Semantic Search Bar (Graph Mode Only) */}
        {viewMode === 'graph' && (
          <form
            onSubmit={handleSearch}
            style={{
              display: 'flex',
              alignItems: 'center',
              background: 'rgba(0, 0, 0, 0.45)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
              borderRadius: 24,
              padding: '4px 14px',
              width: 340,
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
        )}

        {/* Action Controls & Health Badges */}
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          {/* Galaxies & Subsystems Toggle (Graph Mode Only) */}
          {viewMode === 'graph' && (
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
          )}

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
            {health ? (
              <>
                <div title="SurrealDB v2 Status" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
                  <Database size={13} color={health.services.surrealdb.status === 'ok' ? '#34d399' : '#f87171'} />
                  <span style={{ color: 'var(--text-muted)' }}>DB</span>
                </div>
                <div title="HuggingFace TEI Inference" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
                  <Cpu size={13} color={health.services.tei.status === 'ok' ? '#34d399' : '#f87171'} />
                  <span style={{ color: 'var(--text-muted)' }}>TEI</span>
                </div>
              </>
            ) : (
              <div
                title="Omni-Graph backend services are not running. Run 'make omni-graph' in terminal."
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                  background: 'rgba(239, 68, 68, 0.15)',
                  border: '1px solid rgba(239, 68, 68, 0.4)',
                  borderRadius: 12,
                  padding: '2px 8px',
                  fontSize: '0.7rem',
                  color: '#f87171',
                }}
              >
                <span style={{ width: 6, height: 6, borderRadius: '50%', background: '#ef4444' }} />
                <span>Backend Offline</span>
              </div>
            )}
            <div title="Visible Nodes" style={{ display: 'flex', alignItems: 'center', gap: 4, fontSize: '0.72rem' }}>
              <Layers size={13} color="#38bdf8" />
              <span style={{ color: '#38bdf8', fontWeight: 600 }}>{displayNodes.length}</span>
            </div>
          </div>
        </div>
      </header>

      {/* ─── AGENT ANALYTICS & LSP TELEMETRY VIEW ────────────────────── */}
      {viewMode === 'analytics' && (
        <div
          style={{
            position: 'absolute',
            top: 84,
            left: 14,
            right: 14,
            bottom: 14,
            zIndex: 90,
            borderRadius: 12,
            overflow: 'hidden',
          }}
        >
          <AgentAnalytics />
        </div>
      )}

      {/* ─── 3D GRAPH STUDIO VIEW ────────────────────────────────────── */}
      {viewMode === 'graph' && (
        <>
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
            <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', display: 'flex', alignItems: 'center', gap: 6, marginTop: 2 }}>
              <span>{selectedWorkspace ? `Workspace: ${selectedWorkspace}` : 'All Indexed Subsystems'}</span>
              {activeWatchers.some((w) => w.status === 'watching' && (!selectedWorkspace || w.workspace === selectedWorkspace)) && (
                <span
                  title="Dynamic Live Galaxy Re-Clustering active for this workspace (quiescent 3.5s background LPA)"
                  style={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 3,
                    color: 'var(--accent-emerald)',
                    fontSize: '0.62rem',
                    background: 'rgba(52, 211, 153, 0.12)',
                    padding: '1px 5px',
                    borderRadius: 8,
                    border: '1px solid rgba(52, 211, 153, 0.3)',
                    fontWeight: 600,
                  }}
                >
                  <span style={{ width: 4, height: 4, borderRadius: '50%', background: 'var(--accent-emerald)' }} className="pulsing-dot" />
                  Live Sync
                </span>
              )}
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

        {/* View Mode Toggle: Topology Hub vs Flat Tree */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: '1fr 1fr',
            gap: 4,
            background: 'rgba(0, 0, 0, 0.4)',
            padding: 3,
            borderRadius: 8,
            border: '1px solid rgba(255, 255, 255, 0.08)',
          }}
        >
          <button
            onClick={() => setGalaxyViewMode('topology')}
            style={{
              padding: '6px 8px',
              borderRadius: 6,
              border: 'none',
              background: galaxyViewMode === 'topology' ? 'var(--accent-purple)' : 'transparent',
              color: galaxyViewMode === 'topology' ? '#fff' : 'var(--text-muted)',
              fontSize: '0.72rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 5,
              transition: 'all 0.15s ease',
            }}
          >
            <Boxes size={13} />
            <span>Macro Topology</span>
          </button>
          <button
            onClick={() => setGalaxyViewMode('flat')}
            style={{
              padding: '6px 8px',
              borderRadius: 6,
              border: 'none',
              background: galaxyViewMode === 'flat' ? 'var(--accent-cyan)' : 'transparent',
              color: galaxyViewMode === 'flat' ? '#000' : 'var(--text-muted)',
              fontSize: '0.72rem',
              fontWeight: 700,
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: 5,
              transition: 'all 0.15s ease',
            }}
          >
            <Layers size={13} />
            <span>Flat Explorer</span>
          </button>
        </div>

        {galaxyViewMode === 'topology' ? (
          <SubsystemTopologyHub
            galaxies={topologyGalaxies}
            dependencies={topologyDependencies}
            workspace={selectedWorkspace || ''}
            isolatedClusterId={isolatedClusterId}
            onIsolateCluster={(id) => handleToggleIsolateGalaxy(id ?? 0)}
            onFocusCluster={handleFocusCluster}
            onSelectSymbol={(sym) => {
              const target = nodes.find(
                (n) => n.label === sym ||
                       n.label.endsWith(`::${sym}`) ||
                       n.label.endsWith(`.${sym}`) ||
                       n.label.includes(sym)
              );
              const resolved: GraphNode = target || {
                id: `node:symbol:${sym}`,
                label: sym,
                kind: 'symbol',
                language: 'unknown',
                file_path: '',
                workspace: selectedWorkspace || '',
              };
              setSelectedNode(resolved);
              setNodeHistory([]);
            }}
            palette={GALAXY_COLORS}
          />
        ) : (
          <>
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
        <div
          onScroll={(e) => {
            const el = e.currentTarget;
            if (el.scrollHeight - el.scrollTop - el.clientHeight < 250) {
              setVisibleClusterCount((prev) => Math.min(prev + 60, filteredClusters.length));
            }
          }}
          style={{ flex: 1, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 8, paddingRight: 4 }}
        >
          {filteredClusters.slice(0, visibleClusterCount).map((c) => {
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
                  flexShrink: 0,
                  minHeight: 'fit-content',
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
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, overflow: 'hidden', minWidth: 0, flex: 1 }}>
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
                      title={c.name}
                    >
                      {c.name}
                    </span>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0, marginLeft: 8 }}>
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

                      {/* Filtered symbols list - adapts dynamically without nested scroll trap */}
                      {(() => {
                        const isShowingAll = !!expandedAllNodes[c.id];
                        const displayNodes = isShowingAll ? filteredNodes : filteredNodes.slice(0, 30);
                        const hasMore = filteredNodes.length > 30;

                        return (
                          <div
                            style={{
                              display: 'flex',
                              flexDirection: 'column',
                              gap: 3,
                              marginTop: 4,
                            }}
                          >
                            {displayNodes.map((node) => {
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

                            {hasMore && (
                              <button
                                onClick={(e) => {
                                  e.stopPropagation();
                                  setExpandedAllNodes((prev) => ({ ...prev, [c.id]: !isShowingAll }));
                                }}
                                className="cyber-button-secondary"
                                style={{
                                  padding: '4px 8px',
                                  fontSize: '0.7rem',
                                  marginTop: 4,
                                  width: '100%',
                                  justifyContent: 'center',
                                }}
                              >
                                {isShowingAll
                                  ? 'Show less (first 30)'
                                  : `Show all ${filteredNodes.length} symbols (${filteredNodes.length - 30} more)`}
                              </button>
                            )}

                            {filteredNodes.length === 0 && (
                              <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textAlign: 'center', padding: 8 }}>
                                No symbols match current filter.
                              </div>
                            )}
                          </div>
                        );
                      })()}
                    </div>
                  )}
                </div>
              );
            })}

          {filteredClusters.length > visibleClusterCount && (
            <button
              onClick={() => setVisibleClusterCount((prev) => Math.min(prev + 60, filteredClusters.length))}
              className="cyber-button-secondary"
              style={{
                width: '100%',
                justifyContent: 'center',
                padding: '8px 12px',
                fontSize: '0.75rem',
                flexShrink: 0,
                marginTop: 4,
              }}
            >
              <span>Load More ({filteredClusters.length - visibleClusterCount} remaining)</span>
            </button>
          )}

          {clusters.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.8rem', padding: 20 }}>
              No galaxy clusters computed yet. Click <strong>"Compute Galaxy Clusters"</strong> above to partition the AST graph!
            </div>
          )}

          {clusters.length > 0 && filteredClusters.length === 0 && (
            <div style={{ textAlign: 'center', color: 'var(--text-muted)', fontSize: '0.8rem', padding: 20 }}>
              No clusters or symbols match "{clusterSearchTerm}".
            </div>
          )}
        </div>
          </>
        )}
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
              onClick={() => {
                setSelectedNode(null);
                setNodeHistory([]);
              }}
              style={{ background: 'none', border: 'none', color: 'var(--text-muted)', cursor: 'pointer' }}
            >
              <X size={16} />
            </button>
          </div>

          {/* Navigation History & Breadcrumb Bar */}
          {nodeHistory.length > 0 && (
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: 6,
                padding: '8px 10px',
                borderRadius: 8,
                background: 'rgba(56, 189, 248, 0.08)',
                border: '1px solid rgba(56, 189, 248, 0.25)',
                marginTop: -4,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                <button
                  type="button"
                  onClick={() => {
                    const prev = nodeHistory[nodeHistory.length - 1];
                    setNodeHistory((stack) => stack.slice(0, -1));
                    setSelectedNode(prev);
                  }}
                  style={{
                    background: 'none',
                    border: 'none',
                    padding: 0,
                    display: 'flex',
                    alignItems: 'center',
                    gap: 6,
                    color: 'var(--accent-cyan)',
                    fontSize: '0.74rem',
                    fontWeight: 700,
                    cursor: 'pointer',
                  }}
                  title={`Return to previous symbol: ${nodeHistory[nodeHistory.length - 1].label}`}
                >
                  <ArrowLeft size={13} />
                  <span>Back to <code>{nodeHistory[nodeHistory.length - 1].label}</code></span>
                </button>

                <button
                  type="button"
                  onClick={() => {
                    const origin = nodeHistory[0];
                    setNodeHistory([]);
                    setSelectedNode(origin);
                  }}
                  style={{
                    background: 'none',
                    border: 'none',
                    color: 'var(--text-muted)',
                    fontSize: '0.65rem',
                    cursor: 'pointer',
                    textDecoration: 'underline',
                  }}
                  title={`Jump directly back to origin: ${nodeHistory[0].label}`}
                >
                  Origin
                </button>
              </div>

              {/* Breadcrumb path visualization */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 4,
                  fontSize: '0.65rem',
                  color: 'var(--text-muted)',
                  overflowX: 'auto',
                  whiteSpace: 'nowrap',
                  paddingBottom: 2,
                }}
              >
                {nodeHistory.map((item, idx) => (
                  <React.Fragment key={idx}>
                    <span
                      onClick={() => {
                        const target = nodeHistory[idx];
                        setNodeHistory((stack) => stack.slice(0, idx));
                        setSelectedNode(target);
                      }}
                      style={{
                        cursor: 'pointer',
                        color: 'var(--accent-cyan)',
                        maxWidth: 95,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                      }}
                      title={`Jump to ${item.label}`}
                    >
                      {item.label}
                    </span>
                    <span>›</span>
                  </React.Fragment>
                ))}
                <span style={{ color: '#fff', fontWeight: 600, maxWidth: 110, overflow: 'hidden', textOverflow: 'ellipsis' }}>
                  {selectedNode.label}
                </span>
              </div>
            </div>
          )}

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

          {/* Architectural Boundary & Blast Radius Contract (R-030) */}
          <BoundaryContractCard
            symbol={selectedNode.label}
            workspace={selectedNode.workspace || selectedWorkspace || 'tool-scripts'}
            previousSymbol={nodeHistory.length > 0 ? nodeHistory[nodeHistory.length - 1].label : null}
            onNavigateBack={() => {
              if (nodeHistory.length > 0) {
                const prev = nodeHistory[nodeHistory.length - 1];
                setNodeHistory((stack) => stack.slice(0, -1));
                setSelectedNode(prev);
              }
            }}
            onSelectCaller={(caller) => {
              const targetNode = nodes.find(
                (n) => n.label === caller.symbol ||
                       (caller.file_path && n.file_path === caller.file_path && (!caller.line_start || n.line_start === caller.line_start)) ||
                       n.label.endsWith(`::${caller.symbol}`) ||
                       n.label.endsWith(`.${caller.symbol}`)
              );
              const resolvedNode: GraphNode = targetNode || {
                id: `node:${caller.file_path}:${caller.symbol}:${caller.line_start || 1}`,
                label: caller.symbol,
                kind: caller.kind || 'function',
                language: selectedNode.language || 'unknown',
                file_path: caller.file_path,
                line_start: caller.line_start,
                community: caller.galaxy_id,
                workspace: selectedNode.workspace || selectedWorkspace || '',
              };
              setNodeHistory((prev) => [...prev, selectedNode]);
              setSelectedNode(resolvedNode);
            }}
          />

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
        </>
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

      {/* ─── LIVE DELTA SYNC TOAST NOTIFICATION ────────────────────────── */}
      {watchToast && (
        <div
          style={{
            position: 'fixed',
            bottom: activeWatchers.length > 0 ? 80 : 24,
            right: 24,
            zIndex: 10000,
            background: 'rgba(15, 23, 42, 0.94)',
            backdropFilter: 'blur(16px)',
            border: '1px solid rgba(56, 189, 248, 0.45)',
            borderRadius: 10,
            padding: '10px 16px',
            color: '#38bdf8',
            fontSize: '0.78rem',
            fontWeight: 600,
            boxShadow: '0 10px 30px rgba(0, 0, 0, 0.7), 0 0 16px rgba(56, 189, 248, 0.3)',
            display: 'flex',
            alignItems: 'center',
            gap: 10,
            animation: 'fadeIn 0.2s ease-out',
          }}
        >
          <Activity size={16} className="pulsing-dot" color="#38bdf8" />
          <span>{watchToast}</span>
        </div>
      )}

      {/* ─── GLOBAL PERSISTENT LIVE WATCH HUD ───────────────────────────── */}
      {/* Appears across ALL pages (Graph Studio and Agent Analytics) */}
      {activeWatchers.length > 0 && (
        <div
          style={{
            position: 'fixed',
            bottom: 18,
            left: '50%',
            transform: 'translateX(-50%)',
            zIndex: 9998,
            display: 'flex',
            alignItems: 'center',
            gap: 12,
            background: 'rgba(10, 15, 30, 0.92)',
            backdropFilter: 'blur(20px)',
            border: '1px solid rgba(239, 68, 68, 0.4)',
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.8), 0 0 20px rgba(239, 68, 68, 0.2)',
            borderRadius: 30,
            padding: isWatchBarMinimized ? '6px 14px' : '6px 18px',
            transition: 'all 0.3s cubic-bezier(0.16, 1, 0.3, 1)',
          }}
        >
          {/* Live Recording Badge */}
          <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
            <span
              style={{
                width: 9,
                height: 9,
                borderRadius: '50%',
                background: '#ef4444',
                boxShadow: '0 0 10px #ef4444',
                display: 'inline-block',
              }}
              className="pulsing-dot"
            />
            <span
              style={{
                fontSize: '0.72rem',
                fontWeight: 800,
                letterSpacing: '0.08em',
                color: '#f87171',
                textTransform: 'uppercase',
              }}
            >
              Live Recording
            </span>
          </div>

          {!isWatchBarMinimized ? (
            <>
              {/* Active Workspace Chips */}
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  maxWidth: 500,
                  overflowX: 'auto',
                  padding: '2px 0',
                }}
              >
                {activeWatchers.map((w) => (
                  <div
                    key={w.workspace}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 6,
                      background: 'rgba(255, 255, 255, 0.07)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: 16,
                      padding: '3px 10px',
                      fontSize: '0.72rem',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    <Radio size={12} color="#34d399" />
                    <span style={{ fontWeight: 600, color: '#f1f5f9' }}>{w.workspace}</span>
                    <span style={{ color: 'var(--text-muted)', fontSize: '0.66rem' }}>
                      {w.files_tracked} files • {w.events_processed} evts • {w.avg_sync_ms}ms
                    </span>
                    <button
                      type="button"
                      onClick={() => handleStopWatch(w.workspace)}
                      title={`Stop watching ${w.workspace}`}
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-muted)',
                        cursor: 'pointer',
                        padding: 0,
                        marginLeft: 2,
                        display: 'flex',
                        alignItems: 'center',
                      }}
                    >
                      <X size={12} />
                    </button>
                  </div>
                ))}
              </div>

              {/* Actions */}
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, borderLeft: '1px solid rgba(255, 255, 255, 0.15)', paddingLeft: 10 }}>
                <button
                  type="button"
                  onClick={() => setIsWatchModalOpen(true)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 5,
                    background: 'rgba(56, 189, 248, 0.15)',
                    border: '1px solid rgba(56, 189, 248, 0.35)',
                    borderRadius: 12,
                    padding: '3px 9px',
                    color: '#38bdf8',
                    fontSize: '0.7rem',
                    fontWeight: 600,
                    cursor: 'pointer',
                  }}
                >
                  <span>+ Watch More</span>
                </button>
                <button
                  type="button"
                  onClick={() => setIsWatchBarMinimized(true)}
                  title="Minimize live watch HUD"
                  style={{
                    background: 'transparent',
                    border: 'none',
                    color: 'var(--text-muted)',
                    cursor: 'pointer',
                    fontSize: '0.75rem',
                    padding: '2px 4px',
                  }}
                >
                  ─
                </button>
              </div>
            </>
          ) : (
            <button
              type="button"
              onClick={() => setIsWatchBarMinimized(false)}
              style={{
                background: 'transparent',
                border: 'none',
                color: '#38bdf8',
                fontSize: '0.72rem',
                cursor: 'pointer',
                fontWeight: 600,
              }}
            >
              {activeWatchers.length} Active ↗
            </button>
          )}
        </div>
      )}

      {/* ─── WATCH MANAGER MODAL ────────────────────────────────────────── */}
      {isWatchModalOpen && (
        <div
          style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0, 0, 0, 0.85)',
            backdropFilter: 'blur(10px)',
            zIndex: 10001,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 20,
          }}
          onClick={(e) => {
            if (e.target === e.currentTarget) setIsWatchModalOpen(false);
          }}
        >
          <div
            className="glass-panel"
            style={{
              width: '100%',
              maxWidth: 720,
              maxHeight: '85vh',
              background: 'var(--bg-panel)',
              border: '1px solid rgba(56, 189, 248, 0.35)',
              borderRadius: 14,
              overflow: 'hidden',
              display: 'flex',
              flexDirection: 'column',
              boxShadow: '0 25px 60px rgba(0, 0, 0, 0.9), 0 0 30px rgba(56, 189, 248, 0.15)',
            }}
          >
            {/* Modal Header */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '16px 20px',
                borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
                background: 'rgba(0, 0, 0, 0.4)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
                <div
                  style={{
                    width: 34,
                    height: 34,
                    borderRadius: 8,
                    background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.3), rgba(244, 63, 94, 0.3))',
                    border: '1px solid rgba(239, 68, 68, 0.5)',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  <Radio size={18} color="#f87171" />
                </div>
                <div>
                  <div style={{ fontWeight: 700, fontSize: '1rem', color: '#f8fafc', display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span>Dynamic Live Delta Watch Manager</span>
                    {activeWatchers.length > 0 && (
                      <span
                        style={{
                          background: 'rgba(239, 68, 68, 0.25)',
                          color: '#f87171',
                          borderRadius: 12,
                          padding: '2px 8px',
                          fontSize: '0.68rem',
                          fontWeight: 700,
                        }}
                      >
                        {activeWatchers.length} ACTIVE
                      </span>
                    )}
                  </div>
                  <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', marginTop: 2 }}>
                    Real-time FSEvents & inotify daemon: updates AST graphs and vectors on file save
                  </div>
                </div>
              </div>
              <button
                type="button"
                onClick={() => setIsWatchModalOpen(false)}
                className="modal-close-button"
                title="Close Watch Manager"
              >
                <X size={16} />
              </button>
            </div>

            {/* Modal Body */}
            <div style={{ padding: 20, overflowY: 'auto', flex: 1, display: 'flex', flexDirection: 'column', gap: 20 }}>
              {/* Section 1: Active Watchers */}
              <div>
                <div style={{ fontSize: '0.8rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', marginBottom: 10, display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Activity size={14} color="#34d399" />
                  <span>Currently Watched Codebases ({activeWatchers.length})</span>
                </div>

                {activeWatchers.length > 0 ? (
                  <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {activeWatchers.map((w) => (
                      <div
                        key={w.workspace}
                        style={{
                          background: 'rgba(255, 255, 255, 0.04)',
                          border: '1px solid rgba(255, 255, 255, 0.08)',
                          borderRadius: 10,
                          padding: '12px 16px',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'space-between',
                          gap: 16,
                        }}
                      >
                        <div style={{ flex: 1, minWidth: 0 }}>
                          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
                            <span style={{ fontWeight: 700, fontSize: '0.9rem', color: '#f1f5f9' }}>
                              📁 {w.workspace}
                            </span>
                            <span
                              style={{
                                background: w.status === 'syncing' ? 'rgba(251, 191, 36, 0.2)' : 'rgba(52, 211, 153, 0.2)',
                                color: w.status === 'syncing' ? '#fbbf24' : '#34d399',
                                border: `1px solid ${w.status === 'syncing' ? 'rgba(251, 191, 36, 0.4)' : 'rgba(52, 211, 153, 0.4)'}`,
                                borderRadius: 10,
                                padding: '1px 7px',
                                fontSize: '0.64rem',
                                fontWeight: 700,
                                textTransform: 'uppercase',
                              }}
                            >
                              {w.status}
                            </span>
                            {w.cluster_status === 'stale' && (
                              <span
                                style={{
                                  background: 'rgba(244, 63, 94, 0.2)',
                                  color: '#fb7185',
                                  borderRadius: 10,
                                  padding: '1px 6px',
                                  fontSize: '0.62rem',
                                }}
                              >
                                Clusters Stale
                              </span>
                            )}
                          </div>
                          <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', fontFamily: 'monospace', marginBottom: 6, textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap' }}>
                            {w.path}
                          </div>
                          <div style={{ display: 'flex', alignItems: 'center', gap: 14, fontSize: '0.72rem', color: 'var(--text-secondary)' }}>
                            <span>📁 Tracked: <strong style={{ color: '#38bdf8' }}>{w.files_tracked}</strong></span>
                            <span>⚡ Events: <strong style={{ color: '#a855f7' }}>{w.events_processed}</strong></span>
                            <span>✨ Reindexed: <strong style={{ color: '#34d399' }}>{w.files_reindexed}</strong></span>
                            <span>🗑 Deleted: <strong style={{ color: '#f87171' }}>{w.files_deleted}</strong></span>
                            <span>⏱ Latency: <strong style={{ color: '#f59e0b' }}>{w.avg_sync_ms}ms</strong></span>
                          </div>
                        </div>

                        <button
                          type="button"
                          onClick={() => handleStopWatch(w.workspace)}
                          className="cyber-button-secondary"
                          style={{
                            padding: '6px 12px',
                            color: '#f87171',
                            borderColor: 'rgba(239, 68, 68, 0.3)',
                            fontSize: '0.75rem',
                          }}
                        >
                          <Square size={13} color="#f87171" />
                          <span>Stop Watch</span>
                        </button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div
                    style={{
                      background: 'rgba(255, 255, 255, 0.02)',
                      border: '1px dashed rgba(255, 255, 255, 0.1)',
                      borderRadius: 10,
                      padding: 24,
                      textAlign: 'center',
                      color: 'var(--text-muted)',
                      fontSize: '0.8rem',
                    }}
                  >
                    No codebases are currently being live-watched. Start one below to stream live AST changes!
                  </div>
                )}
              </div>

              {/* Section 2: Start New Live Watcher */}
              <div
                style={{
                  background: 'rgba(0, 0, 0, 0.3)',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  borderRadius: 10,
                  padding: 16,
                }}
              >
                <div style={{ fontSize: '0.8rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.05em', color: 'var(--text-secondary)', marginBottom: 12, display: 'flex', alignItems: 'center', gap: 6 }}>
                  <Play size={14} color="#38bdf8" />
                  <span>Start Live Watcher</span>
                </div>

                {/* Quick Select from Ingested Workspaces */}
                {workspaces.length > 0 && (
                  <div style={{ marginBottom: 14 }}>
                    <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)', marginBottom: 6 }}>
                      Quick Select from Ingested Codebases:
                    </div>
                    <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6 }}>
                      {workspaces.map((ws) => {
                        const isAlreadyWatched = activeWatchers.some((w) => w.workspace === ws.workspace);
                        return (
                          <button
                            key={ws.workspace}
                            type="button"
                            disabled={isAlreadyWatched || isStartingWatch}
                            onClick={() => {
                              const targetP =
                                ws.root_path ||
                                (ws.workspace === 'tool-scripts' || ws.workspace === 'workspace'
                                  ? '/workspace'
                                  : ws.workspace === 'session-explorer' || ws.workspace === 'omni-graph'
                                  ? `/workspace/tools/${ws.workspace}`
                                  : `/Users/aparv/Library/CloudStorage/OneDrive-Personal/G-Drive/Interviews/knowledge/${ws.workspace}`);
                              setWatchTargetFolder(targetP);
                              setWatchCustomProject(ws.workspace);
                              handleStartWatch(targetP, ws.workspace, watchDebounceMs);
                            }}
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: 8,
                              padding: '6px 12px',
                              borderRadius: 8,
                              border: isAlreadyWatched
                                ? '1px solid rgba(52, 211, 153, 0.4)'
                                : '1px solid rgba(255, 255, 255, 0.1)',
                              background: isAlreadyWatched
                                ? 'rgba(52, 211, 153, 0.12)'
                                : 'rgba(255, 255, 255, 0.04)',
                              color: isAlreadyWatched ? '#34d399' : '#e2e8f0',
                              fontSize: '0.74rem',
                              cursor: isAlreadyWatched ? 'default' : 'pointer',
                            }}
                          >
                            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start' }}>
                              <span style={{ fontWeight: 600 }}>📁 {ws.workspace}</span>
                              <span style={{ fontSize: '0.62rem', color: 'var(--text-muted)', fontFamily: 'monospace' }}>
                                {ws.root_path || (ws.workspace === 'tool-scripts' ? '/workspace' : `.../${ws.workspace}`)}
                              </span>
                            </div>
                            {isAlreadyWatched ? (
                              <span style={{ fontSize: '0.62rem', color: '#34d399', fontWeight: 700 }}>WATCHING</span>
                            ) : (
                              <span style={{ fontSize: '0.62rem', color: '#38bdf8' }}>+ Watch</span>
                            )}
                          </button>
                        );
                      })}
                    </div>
                  </div>
                )}

                {/* Custom Folder Path Input */}
                <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                  <div>
                    <label style={{ display: 'block', fontSize: '0.72rem', color: '#94a3b8', marginBottom: 4, fontWeight: 600 }}>
                      Directory Path to Watch (Absolute or Container Path):
                    </label>
                    <input
                      type="text"
                      value={watchTargetFolder}
                      onChange={(e) => setWatchTargetFolder(e.target.value)}
                      placeholder="/workspace/path/to/repo"
                      className="cyber-input"
                      style={{
                        width: '100%',
                        fontSize: '0.82rem',
                        background: '#0f172a',
                        color: '#f8fafc',
                        border: '1px solid rgba(255, 255, 255, 0.25)',
                        borderRadius: 8,
                        padding: '8px 12px',
                        colorScheme: 'dark',
                      }}
                    />
                  </div>

                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 10 }}>
                    <div>
                      <label style={{ display: 'block', fontSize: '0.72rem', color: '#94a3b8', marginBottom: 4, fontWeight: 600 }}>
                        Workspace Alias (Optional):
                      </label>
                      <input
                        type="text"
                        value={watchCustomProject}
                        onChange={(e) => setWatchCustomProject(e.target.value)}
                        placeholder="e.g. my-app"
                        className="cyber-input"
                        style={{
                          width: '100%',
                          fontSize: '0.82rem',
                          background: '#0f172a',
                          color: '#f8fafc',
                          border: '1px solid rgba(255, 255, 255, 0.25)',
                          borderRadius: 8,
                          padding: '8px 12px',
                          colorScheme: 'dark',
                        }}
                      />
                    </div>
                    <div>
                      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 4 }}>
                        <label style={{ fontSize: '0.72rem', color: '#94a3b8', fontWeight: 600 }}>
                          Debounce Window:
                        </label>
                        <span style={{ fontSize: '0.68rem', color: '#38bdf8', fontWeight: 600 }}>
                          {watchDebounceMs}ms selected
                        </span>
                      </div>
                      <select
                        value={watchDebounceMs}
                        onChange={(e) => setWatchDebounceMs(Number(e.target.value))}
                        className="cyber-select"
                        style={{
                          width: '100%',
                          fontSize: '0.82rem',
                          background: '#0f172a',
                          color: '#f8fafc',
                          border: '1px solid rgba(255, 255, 255, 0.25)',
                          borderRadius: 8,
                          padding: '8px 12px',
                          colorScheme: 'dark',
                          cursor: 'pointer',
                        }}
                      >
                        <option value={200} style={{ background: '#0f172a', color: '#f8fafc' }}>
                          ⚡ 200ms (Ultra-fast)
                        </option>
                        <option value={500} style={{ background: '#0f172a', color: '#38bdf8', fontWeight: 700 }}>
                          ⭐ 500ms (Recommended)
                        </option>
                        <option value={1000} style={{ background: '#0f172a', color: '#f8fafc' }}>
                          ⏱ 1000ms (1 second)
                        </option>
                        <option value={2000} style={{ background: '#0f172a', color: '#f8fafc' }}>
                          ⏱ 2000ms (2 seconds)
                        </option>
                      </select>
                    </div>
                  </div>

                  <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: 14 }}>
                    <button
                      type="button"
                      disabled={isStartingWatch || !watchTargetFolder.trim()}
                      onClick={() => handleStartWatch(watchTargetFolder, watchCustomProject, watchDebounceMs)}
                      className="cyber-button"
                      style={{ padding: '8px 20px', fontSize: '0.82rem' }}
                    >
                      {isStartingWatch ? <RefreshCw size={14} className="pulsing-dot" /> : <Radio size={14} />}
                      <span>{isStartingWatch ? 'Starting Watcher...' : 'Start Live Watch'}</span>
                    </button>
                  </div>
                </div>
              </div>
            </div>

            {/* Modal Footer Bar */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '12px 20px',
                borderTop: '1px solid rgba(255, 255, 255, 0.08)',
                background: 'rgba(0, 0, 0, 0.45)',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: '0.74rem', color: 'var(--text-muted)' }}>
                <span
                  style={{
                    display: 'inline-block',
                    width: 7,
                    height: 7,
                    borderRadius: '50%',
                    background: activeWatchers.length > 0 ? '#34d399' : '#64748b',
                    boxShadow: activeWatchers.length > 0 ? '0 0 8px #34d399' : 'none',
                  }}
                />
                <span>
                  {activeWatchers.length > 0
                    ? `${activeWatchers.length} active watcher${activeWatchers.length === 1 ? '' : 's'} registered in daemon`
                    : 'File watcher daemon idle'}
                </span>
              </div>

              <button
                type="button"
                onClick={() => setIsWatchModalOpen(false)}
                className="cyber-button-secondary"
                style={{
                  padding: '7px 20px',
                  fontSize: '0.8rem',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 6,
                }}
              >
                <span>Done</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
