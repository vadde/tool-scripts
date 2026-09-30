import React, { useState, useMemo, useEffect } from 'react';
import {
  Maximize2,
  Minimize2,
  Search,
  Zap,
} from 'lucide-react';

export interface GalaxyRecord {
  workspace: string;
  galaxy_id: number;
  name: string;
  dominant_path: string;
  node_count: number;
  internal_edges: number;
  external_edges: number;
  afferent_coupling: number; // Ca
  efferent_coupling: number; // Ce
  instability: number; // I
  role: string; // 'Core Foundation' | 'Domain Service' | 'Orchestrator / Leaf'
  key_symbols: string[];
  languages: string[];
  updated_at?: string | null;
}

export interface GalaxyDependencyEdge {
  from_galaxy: number;
  to_galaxy: number;
  source_symbol: string;
  target_symbol: string;
  edge_type: string;
}

interface SubsystemTopologyHubProps {
  galaxies: GalaxyRecord[];
  dependencies: GalaxyDependencyEdge[];
  workspace: string;
  isolatedClusterId: number | null;
  onIsolateCluster: (galaxyId: number | null) => void;
  onFocusCluster: (galaxyId: number) => void;
  onSelectSymbol?: (symbolName: string) => void;
  palette: string[];
}

export const SubsystemTopologyHub: React.FC<SubsystemTopologyHubProps> = ({
  galaxies,
  dependencies,
  workspace,
  isolatedClusterId,
  onIsolateCluster,
  onFocusCluster,
  onSelectSymbol,
  palette,
}) => {
  const [roleFilter, setRoleFilter] = useState<'all' | 'Core Foundation' | 'Domain Service' | 'Orchestrator / Leaf'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [sortBy, setSortBy] = useState<'nodes' | 'instability_high' | 'instability_low' | 'edges'>('nodes');
  const [expandedGalaxyId, setExpandedGalaxyId] = useState<number | null>(null);
  const [visibleCount, setVisibleCount] = useState(40);

  // Reset pagination on filter, search, sort, or workspace changes
  useEffect(() => {
    setVisibleCount(40);
  }, [roleFilter, searchQuery, sortBy, workspace]);

  // Global counts
  const stats = useMemo(() => {
    let coreCount = 0;
    let domainCount = 0;
    let leafCount = 0;

    galaxies.forEach((g) => {
      if (g.role === 'Core Foundation') coreCount++;
      else if (g.role === 'Domain Service') domainCount++;
      else leafCount++;
    });

    return {
      total: galaxies.length,
      core: coreCount,
      domain: domainCount,
      leaf: leafCount,
      deps: dependencies.length,
    };
  }, [galaxies, dependencies]);

  // Filtering & sorting
  const filteredGalaxies = useMemo(() => {
    return galaxies
      .filter((g) => {
        const matchesRole = roleFilter === 'all' || g.role === roleFilter;
        const q = searchQuery.toLowerCase().trim();
        const matchesSearch =
          !q ||
          g.name.toLowerCase().includes(q) ||
          g.dominant_path.toLowerCase().includes(q) ||
          g.key_symbols.some((s) => s.toLowerCase().includes(q));
        return matchesRole && matchesSearch;
      })
      .sort((a, b) => {
        if (sortBy === 'nodes') return b.node_count - a.node_count;
        if (sortBy === 'instability_high') return b.instability - a.instability;
        if (sortBy === 'instability_low') return a.instability - b.instability;
        if (sortBy === 'edges') return (b.afferent_coupling + b.efferent_coupling) - (a.afferent_coupling + a.efferent_coupling);
        return 0;
      });
  }, [galaxies, roleFilter, searchQuery, sortBy]);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10, flex: 1, minHeight: 0 }}>
      {/* Metric Cards Row */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 6, flexShrink: 0 }}>
        <div
          onClick={() => setRoleFilter('all')}
          style={{
            padding: '8px 10px',
            borderRadius: 8,
            background: roleFilter === 'all' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(255, 255, 255, 0.03)',
            border: `1px solid ${roleFilter === 'all' ? 'var(--accent-cyan)' : 'rgba(255, 255, 255, 0.06)'}`,
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
        >
          <div style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>TOTAL GALAXIES</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-cyan)' }}>{stats.total}</div>
        </div>

        <div
          onClick={() => setRoleFilter('Core Foundation')}
          style={{
            padding: '8px 10px',
            borderRadius: 8,
            background: roleFilter === 'Core Foundation' ? 'rgba(192, 132, 252, 0.15)' : 'rgba(255, 255, 255, 0.03)',
            border: `1px solid ${roleFilter === 'Core Foundation' ? 'var(--accent-purple)' : 'rgba(255, 255, 255, 0.06)'}`,
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
          title="Stable subsystems (I <= 0.25). Heavily depended on by others."
        >
          <div style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>CORE (STABLE)</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-purple)' }}>{stats.core}</div>
        </div>

        <div
          onClick={() => setRoleFilter('Domain Service')}
          style={{
            padding: '8px 10px',
            borderRadius: 8,
            background: roleFilter === 'Domain Service' ? 'rgba(56, 189, 248, 0.15)' : 'rgba(255, 255, 255, 0.03)',
            border: `1px solid ${roleFilter === 'Domain Service' ? 'var(--accent-blue)' : 'rgba(255, 255, 255, 0.06)'}`,
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
          title="Balanced business logic domain services (0.25 < I <= 0.65)"
        >
          <div style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>DOMAIN SERVICES</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-blue)' }}>{stats.domain}</div>
        </div>

        <div
          onClick={() => setRoleFilter('Orchestrator / Leaf')}
          style={{
            padding: '8px 10px',
            borderRadius: 8,
            background: roleFilter === 'Orchestrator / Leaf' ? 'rgba(251, 191, 36, 0.15)' : 'rgba(255, 255, 255, 0.03)',
            border: `1px solid ${roleFilter === 'Orchestrator / Leaf' ? 'var(--accent-amber)' : 'rgba(255, 255, 255, 0.06)'}`,
            cursor: 'pointer',
            transition: 'all 0.15s ease',
          }}
          title="High instability leaves (I > 0.65). Call outward, easy to refactor."
        >
          <div style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>LEAVES / ORCH</div>
          <div style={{ fontSize: '1rem', fontWeight: 700, color: 'var(--accent-amber)' }}>{stats.leaf}</div>
        </div>
      </div>

      {/* Search & Sort Controls */}
      <div style={{ display: 'flex', gap: 6, alignItems: 'center', flexShrink: 0 }}>
        <div style={{ position: 'relative', flex: 1 }}>
          <Search
            size={13}
            color="var(--text-muted)"
            style={{ position: 'absolute', left: 8, top: '50%', transform: 'translateY(-50%)' }}
          />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={`Search ${workspace ? `'${workspace}'` : ''} subsystems, paths, symbols...`}
            style={{
              width: '100%',
              padding: '6px 8px 6px 26px',
              borderRadius: 6,
              background: 'rgba(0, 0, 0, 0.35)',
              border: '1px solid rgba(255, 255, 255, 0.08)',
              color: '#fff',
              fontSize: '0.72rem',
              outline: 'none',
            }}
          />
        </div>

        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as any)}
          style={{
            padding: '6px 8px',
            borderRadius: 6,
            background: 'rgba(0, 0, 0, 0.45)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
            color: 'var(--text-secondary)',
            fontSize: '0.7rem',
            cursor: 'pointer',
            outline: 'none',
          }}
        >
          <option value="nodes">Largest Subsystems</option>
          <option value="instability_low">Most Stable (Low I)</option>
          <option value="instability_high">Most Volatile (High I)</option>
          <option value="edges">Most Cross-Boundary Links</option>
        </select>
      </div>

      {/* Subsystem Cards List */}
      <div
        onScroll={(e) => {
          const el = e.currentTarget;
          if (el.scrollHeight - el.scrollTop - el.clientHeight < 250) {
            setVisibleCount((prev) => Math.min(prev + 50, filteredGalaxies.length));
          }
        }}
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: 8,
          flex: 1,
          minHeight: 0,
          overflowY: 'auto',
          paddingRight: 2,
        }}
      >
        {filteredGalaxies.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '24px 12px', color: 'var(--text-muted)', fontSize: '0.75rem' }}>
            No architectural subsystems match the current filters.
          </div>
        ) : (
          filteredGalaxies.slice(0, visibleCount).map((g) => {
            const color = palette[Math.abs(g.galaxy_id) % palette.length];
            const isIsolated = isolatedClusterId === g.galaxy_id;
            const isExpanded = expandedGalaxyId === g.galaxy_id;
            const instPct = Math.round(g.instability * 100);

            // Role styling
            const roleColor =
              g.role === 'Core Foundation'
                ? 'var(--accent-purple)'
                : g.role === 'Domain Service'
                ? 'var(--accent-blue)'
                : 'var(--accent-amber)';

            return (
              <div
                key={g.galaxy_id}
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
                      ? 'rgba(56, 189, 248, 0.35)'
                      : 'rgba(255, 255, 255, 0.06)'
                  }`,
                  overflow: 'hidden',
                  transition: 'all 0.18s ease',
                }}
              >
                {/* Header row */}
                <div
                  onClick={() => setExpandedGalaxyId(isExpanded ? null : g.galaxy_id)}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '8px 10px',
                    cursor: 'pointer',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0, flex: 1 }}>
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
                    <div style={{ overflow: 'hidden', minWidth: 0 }}>
                      <div
                        style={{
                          fontSize: '0.78rem',
                          fontWeight: 700,
                          color: '#fff',
                          overflow: 'hidden',
                          textOverflow: 'ellipsis',
                          whiteSpace: 'nowrap',
                        }}
                        title={g.name}
                      >
                        Galaxy #{g.galaxy_id}: {g.name}
                      </div>
                      <div style={{ fontSize: '0.65rem', color: 'var(--text-muted)' }}>
                        {g.dominant_path}
                      </div>
                    </div>
                  </div>

                  <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexShrink: 0, marginLeft: 8 }}>
                    <span
                      style={{
                        fontSize: '0.65rem',
                        fontWeight: 700,
                        padding: '2px 6px',
                        borderRadius: 4,
                        backgroundColor: `${roleColor}22`,
                        color: roleColor,
                        border: `1px solid ${roleColor}44`,
                      }}
                    >
                      {g.role === 'Core Foundation' ? 'Core' : g.role === 'Domain Service' ? 'Domain' : 'Leaf'}
                    </span>
                    <span
                      style={{
                        fontSize: '0.68rem',
                        color: 'var(--text-secondary)',
                        background: 'rgba(255, 255, 255, 0.06)',
                        borderRadius: 4,
                        padding: '2px 6px',
                      }}
                    >
                      {g.node_count} nodes
                    </span>
                  </div>
                </div>

                {/* Instability Progress Bar */}
                <div style={{ padding: '0 10px 8px 10px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.64rem', marginBottom: 3 }}>
                    <span style={{ color: 'var(--text-muted)' }}>Instability (I = Ce / (Ca + Ce))</span>
                    <strong style={{ color: instPct > 65 ? 'var(--accent-amber)' : instPct < 30 ? 'var(--accent-emerald)' : 'var(--accent-cyan)' }}>
                      {instPct}% ({g.instability.toFixed(2)})
                    </strong>
                  </div>
                  <div
                    style={{
                      height: 4,
                      borderRadius: 2,
                      background: 'rgba(255, 255, 255, 0.08)',
                      overflow: 'hidden',
                      position: 'relative',
                    }}
                  >
                    <div
                      style={{
                        height: '100%',
                        width: `${instPct}%`,
                        background: `linear-gradient(90deg, #34d399 0%, #60a5fa 50%, #fbbf24 85%, #fb7185 100%)`,
                        borderRadius: 2,
                      }}
                    />
                  </div>
                </div>

                {/* Expanded Details */}
                {isExpanded && (
                  <div
                    style={{
                      padding: '8px 10px 10px 10px',
                      borderTop: '1px solid rgba(255, 255, 255, 0.06)',
                      background: 'rgba(0, 0, 0, 0.25)',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 8,
                    }}
                  >
                    {/* Coupling breakdown */}
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: 4, textAlign: 'center' }}>
                      <div style={{ padding: 4, background: 'rgba(255, 255, 255, 0.03)', borderRadius: 4 }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)' }}>Afferent (Ca)</div>
                        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--accent-emerald)' }}>{g.afferent_coupling} in</div>
                      </div>
                      <div style={{ padding: 4, background: 'rgba(255, 255, 255, 0.03)', borderRadius: 4 }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)' }}>Efferent (Ce)</div>
                        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--accent-amber)' }}>{g.efferent_coupling} out</div>
                      </div>
                      <div style={{ padding: 4, background: 'rgba(255, 255, 255, 0.03)', borderRadius: 4 }}>
                        <div style={{ fontSize: '0.6rem', color: 'var(--text-muted)' }}>Internal Links</div>
                        <div style={{ fontSize: '0.8rem', fontWeight: 700, color: 'var(--accent-cyan)' }}>{g.internal_edges}</div>
                      </div>
                    </div>

                    {/* Key Symbols Chips */}
                    {g.key_symbols && g.key_symbols.length > 0 && (
                      <div>
                        <div style={{ fontSize: '0.64rem', color: 'var(--text-muted)', marginBottom: 4 }}>KEY EXPORTED SYMBOLS</div>
                        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 4 }}>
                          {g.key_symbols.map((sym, sIdx) => (
                            <span
                              key={sIdx}
                              onClick={() => onSelectSymbol?.(sym)}
                              style={{
                                fontSize: '0.65rem',
                                padding: '2px 6px',
                                borderRadius: 4,
                                background: 'rgba(255, 255, 255, 0.06)',
                                color: '#e2e8f0',
                                cursor: onSelectSymbol ? 'pointer' : 'default',
                                border: '1px solid rgba(255, 255, 255, 0.08)',
                              }}
                            >
                              <code>{sym}</code>
                            </span>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* Action Buttons */}
                    <div style={{ display: 'flex', gap: 6, marginTop: 4 }}>
                      <button
                        onClick={() => onIsolateCluster(isIsolated ? null : g.galaxy_id)}
                        style={{
                          flex: 1,
                          padding: '5px 8px',
                          borderRadius: 6,
                          background: isIsolated ? 'rgba(168, 85, 247, 0.4)' : 'rgba(168, 85, 247, 0.2)',
                          border: '1px solid rgba(168, 85, 247, 0.5)',
                          color: '#fff',
                          fontSize: '0.7rem',
                          fontWeight: 600,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          gap: 4,
                        }}
                      >
                        {isIsolated ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
                        <span>{isIsolated ? 'Show Omniverse' : 'Isolate in 3D'}</span>
                      </button>

                      <button
                        onClick={() => onFocusCluster(g.galaxy_id)}
                        style={{
                          flex: 1,
                          padding: '5px 8px',
                          borderRadius: 6,
                          background: 'rgba(56, 189, 248, 0.15)',
                          border: '1px solid rgba(56, 189, 248, 0.4)',
                          color: 'var(--accent-cyan)',
                          fontSize: '0.7rem',
                          fontWeight: 600,
                          cursor: 'pointer',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                          gap: 4,
                        }}
                      >
                        <Zap size={12} />
                        <span>Focus View</span>
                      </button>
                    </div>
                  </div>
                )}
              </div>
            );
          })
        )}

        {filteredGalaxies.length > visibleCount && (
          <button
            type="button"
            onClick={() => setVisibleCount((prev) => Math.min(prev + 50, filteredGalaxies.length))}
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
            <span>Load More ({filteredGalaxies.length - visibleCount} remaining)</span>
          </button>
        )}
      </div>
    </div>
  );
};

export default SubsystemTopologyHub;
