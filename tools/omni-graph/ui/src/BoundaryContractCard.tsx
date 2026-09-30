import React, { useEffect, useState } from 'react';
import {
  ShieldAlert,
  ShieldCheck,
  Shield,
  AlertTriangle,
  ExternalLink,
  ChevronDown,
  ChevronRight,
  Copy,
  Check,
  Activity,
  Boxes,
} from 'lucide-react';

export interface CallerRef {
  symbol: string;
  kind: string;
  file_path: string;
  line_start?: number;
  galaxy_id?: number;
  galaxy_name?: string;
}

export interface SymbolBoundaryInfo {
  symbol: string;
  kind: string;
  file_path: string;
  line_start?: number;
  workspace?: string;
  home_galaxy?: {
    id: number;
    name: string;
    dominant_path: string;
    instability: number;
    role: string;
  } | null;
  containment: {
    is_exported: boolean;
    internal_callers_count: number;
    cross_galaxy_callers_count: number;
    architectural_status: string; // 'ISOLATED' | 'CONTAINED_INTERNAL' | 'BOUNDARY_CROSSING'
  };
  internal_callers: CallerRef[];
  cross_galaxy_callers: CallerRef[];
  agent_actionable_advice: {
    risk_level: 'LOW' | 'MEDIUM' | 'HIGH';
    summary: string;
    rule_of_thumb: string;
  };
}

interface BoundaryContractCardProps {
  symbol: string;
  workspace: string;
  onSelectCaller?: (caller: CallerRef) => void;
}

export const BoundaryContractCard: React.FC<BoundaryContractCardProps> = ({
  symbol,
  workspace,
  onSelectCaller,
}) => {
  const [data, setData] = useState<SymbolBoundaryInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showInternal, setShowInternal] = useState(false);
  const [copiedContract, setCopiedContract] = useState(false);

  useEffect(() => {
    if (!symbol) return;
    let isCancelled = false;
    setLoading(true);
    setError(null);

    fetch(`/api/galaxy/boundary?symbol=${encodeURIComponent(symbol)}&workspace=${encodeURIComponent(workspace || '')}`)
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
      })
      .then((res) => {
        if (isCancelled) return;
        if (res.error) {
          setError(res.error);
          setData(null);
        } else {
          setData(res);
        }
      })
      .catch((err) => {
        if (!isCancelled) {
          setError(err.message || 'Failed to load boundary');
          setData(null);
        }
      })
      .finally(() => {
        if (!isCancelled) setLoading(false);
      });

    return () => {
      isCancelled = true;
    };
  }, [symbol, workspace]);

  if (loading) {
    return (
      <div
        style={{
          padding: '12px 14px',
          background: 'rgba(15, 23, 42, 0.65)',
          borderRadius: 10,
          border: '1px solid rgba(56, 189, 248, 0.2)',
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          fontSize: '0.75rem',
          color: 'var(--text-secondary)',
        }}
      >
        <Activity size={14} className="pulsing-dot" color="var(--accent-cyan)" />
        <span>Evaluating architectural boundary & blast radius...</span>
      </div>
    );
  }

  if (error || !data) {
    return null;
  }

  const { containment, home_galaxy, agent_actionable_advice } = data;
  const isBoundaryCrossing = containment.architectural_status === 'BOUNDARY_CROSSING';
  const isInternal = containment.architectural_status === 'CONTAINED_INTERNAL';

  // Format contract prompt for LLM consumption
  const handleCopyAgentContract = () => {
    const foreignList = data.cross_galaxy_callers
      .map((c) => `- \`${c.symbol}\` in \`${c.file_path}:${c.line_start || 1}\` (Galaxy: ${c.galaxy_name || 'external'})`)
      .join('\n');

    const promptContract = `### 🛡️ Omni-Graph Architectural Boundary Contract
**Symbol**: \`${data.symbol}\` (${data.kind})
**Subsystem (Home Galaxy)**: #${home_galaxy?.id ?? 'N/A'} • ${home_galaxy?.name ?? 'Unknown'} (${home_galaxy?.role ?? 'Domain'})
**Instability Metric (I)**: ${home_galaxy ? (home_galaxy.instability * 100).toFixed(1) + '%' : 'N/A'}
**Containment Status**: ${containment.architectural_status}
**Risk Level**: ${agent_actionable_advice.risk_level}

#### Blast Radius Summary:
${agent_actionable_advice.summary}

#### Architectural Rule-of-Thumb:
${agent_actionable_advice.rule_of_thumb}

#### Cross-Boundary Foreign Callers (${data.cross_galaxy_callers.length}):
${foreignList || '_None (contained within home subsystem)_'}
`;
    navigator.clipboard.writeText(promptContract).then(() => {
      setCopiedContract(true);
      setTimeout(() => setCopiedContract(false), 2500);
    });
  };

  const statusBorder = isBoundaryCrossing
    ? 'rgba(251, 113, 133, 0.4)'
    : isInternal
    ? 'rgba(52, 211, 153, 0.4)'
    : 'rgba(148, 163, 184, 0.2)';

  const statusBg = isBoundaryCrossing
    ? 'linear-gradient(145deg, rgba(244, 63, 94, 0.12), rgba(15, 23, 42, 0.8))'
    : isInternal
    ? 'linear-gradient(145deg, rgba(16, 185, 129, 0.12), rgba(15, 23, 42, 0.8))'
    : 'linear-gradient(145deg, rgba(148, 163, 184, 0.08), rgba(15, 23, 42, 0.8))';

  const riskColor =
    agent_actionable_advice.risk_level === 'HIGH'
      ? 'var(--accent-rose)'
      : agent_actionable_advice.risk_level === 'MEDIUM'
      ? 'var(--accent-amber)'
      : 'var(--accent-emerald)';

  return (
    <div
      style={{
        borderRadius: 10,
        padding: 12,
        background: statusBg,
        border: `1px solid ${statusBorder}`,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        boxShadow: '0 8px 24px -8px rgba(0, 0, 0, 0.5)',
      }}
    >
      {/* Top Header: Containment Badge */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          {isBoundaryCrossing ? (
            <ShieldAlert size={15} color="var(--accent-rose)" />
          ) : isInternal ? (
            <ShieldCheck size={15} color="var(--accent-emerald)" />
          ) : (
            <Shield size={15} color="var(--text-muted)" />
          )}
          <span
            style={{
              fontSize: '0.72rem',
              fontWeight: 700,
              textTransform: 'uppercase',
              letterSpacing: '0.04em',
              color: isBoundaryCrossing
                ? 'var(--accent-rose)'
                : isInternal
                ? 'var(--accent-emerald)'
                : 'var(--text-secondary)',
            }}
          >
            {isBoundaryCrossing
              ? 'Boundary Crossing'
              : isInternal
              ? 'Contained Internal'
              : 'Isolated Node'}
          </span>
        </div>

        {/* Risk Badge */}
        <span
          style={{
            fontSize: '0.65rem',
            fontWeight: 800,
            padding: '2px 7px',
            borderRadius: 4,
            backgroundColor: `${riskColor}22`,
            color: riskColor,
            border: `1px solid ${riskColor}55`,
          }}
        >
          {agent_actionable_advice.risk_level} RISK
        </span>
      </div>

      {/* Subsystem / Home Galaxy Context */}
      {home_galaxy && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '6px 8px',
            borderRadius: 6,
            background: 'rgba(255, 255, 255, 0.04)',
            fontSize: '0.72rem',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, minWidth: 0, flex: 1 }}>
            <Boxes size={13} color="var(--accent-purple)" style={{ flexShrink: 0 }} />
            <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              <span style={{ color: 'var(--text-muted)' }}>Galaxy #{home_galaxy.id}: </span>
              <strong style={{ color: '#fff' }}>{home_galaxy.name}</strong>
            </div>
          </div>
          <span
            style={{
              fontSize: '0.65rem',
              padding: '2px 6px',
              borderRadius: 4,
              backgroundColor: 'rgba(192, 132, 252, 0.15)',
              color: 'var(--accent-purple)',
              fontWeight: 600,
              flexShrink: 0,
              marginLeft: 6,
            }}
          >
            {home_galaxy.role}
          </span>
        </div>
      )}

      {/* Coupling Stats Row */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
        <div
          style={{
            padding: '6px 8px',
            borderRadius: 6,
            background: 'rgba(0, 0, 0, 0.35)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <span style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>Internal Callers</span>
          <span style={{ fontSize: '0.9rem', fontWeight: 700, color: 'var(--accent-emerald)' }}>
            {containment.internal_callers_count}
          </span>
        </div>

        <div
          style={{
            padding: '6px 8px',
            borderRadius: 6,
            background: 'rgba(0, 0, 0, 0.35)',
            border: '1px solid rgba(255, 255, 255, 0.05)',
            display: 'flex',
            flexDirection: 'column',
          }}
        >
          <span style={{ fontSize: '0.62rem', color: 'var(--text-muted)' }}>Cross-Galaxy Callers</span>
          <span
            style={{
              fontSize: '0.9rem',
              fontWeight: 700,
              color: containment.cross_galaxy_callers_count > 0 ? 'var(--accent-rose)' : 'var(--text-muted)',
            }}
          >
            {containment.cross_galaxy_callers_count}
          </span>
        </div>
      </div>

      {/* Agent Actionable Advisory Banner */}
      <div
        style={{
          padding: '8px 10px',
          borderRadius: 6,
          backgroundColor: 'rgba(0, 0, 0, 0.4)',
          borderLeft: `3px solid ${riskColor}`,
          fontSize: '0.7rem',
          lineHeight: 1.45,
          color: '#e2e8f0',
        }}
      >
        <div style={{ fontWeight: 600, marginBottom: 3, color: riskColor, display: 'flex', alignItems: 'center', gap: 4 }}>
          <AlertTriangle size={12} />
          <span>Architectural Advisory:</span>
        </div>
        <div style={{ color: 'var(--text-secondary)', marginBottom: 4 }}>{agent_actionable_advice.summary}</div>
        <div style={{ color: '#cbd5e1', fontStyle: 'italic', fontSize: '0.68rem' }}>
          {agent_actionable_advice.rule_of_thumb}
        </div>
      </div>

      {/* Foreign Callers List */}
      {data.cross_galaxy_callers.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div style={{ fontSize: '0.68rem', fontWeight: 700, color: 'var(--accent-rose)', textTransform: 'uppercase' }}>
            Foreign Callers Across Subsystems ({data.cross_galaxy_callers.length})
          </div>
          <div style={{ maxHeight: 110, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 4, paddingRight: 2 }}>
            {data.cross_galaxy_callers.map((caller, idx) => (
              <div
                key={idx}
                onClick={() => onSelectCaller?.(caller)}
                style={{
                  padding: '4px 6px',
                  borderRadius: 4,
                  background: 'rgba(244, 63, 94, 0.08)',
                  border: '1px solid rgba(244, 63, 94, 0.2)',
                  fontSize: '0.68rem',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  cursor: onSelectCaller ? 'pointer' : 'default',
                  transition: 'background 0.15s ease',
                }}
              >
                <div style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minWidth: 0, flex: 1 }}>
                  <code style={{ color: '#fff', fontWeight: 600 }}>{caller.symbol}</code>
                  <span style={{ color: 'var(--text-muted)', marginLeft: 6 }}>
                    ({caller.galaxy_name || `Galaxy #${caller.galaxy_id}`})
                  </span>
                </div>
                {onSelectCaller && <ExternalLink size={11} color="var(--accent-rose)" style={{ flexShrink: 0, marginLeft: 4 }} />}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Collapsible Internal Callers */}
      {data.internal_callers.length > 0 && (
        <div>
          <button
            onClick={() => setShowInternal(!showInternal)}
            style={{
              background: 'none',
              border: 'none',
              padding: 0,
              display: 'flex',
              alignItems: 'center',
              gap: 4,
              fontSize: '0.68rem',
              color: 'var(--text-muted)',
              cursor: 'pointer',
            }}
          >
            {showInternal ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            <span>Internal Callers ({data.internal_callers.length})</span>
          </button>

          {showInternal && (
            <div style={{ maxHeight: 90, overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: 3, marginTop: 4 }}>
              {data.internal_callers.map((caller, idx) => (
                <div
                  key={idx}
                  onClick={() => onSelectCaller?.(caller)}
                  style={{
                    padding: '3px 6px',
                    borderRadius: 4,
                    background: 'rgba(255, 255, 255, 0.03)',
                    fontSize: '0.67rem',
                    color: 'var(--text-secondary)',
                    cursor: onSelectCaller ? 'pointer' : 'default',
                  }}
                >
                  <code>{caller.symbol}</code> <span style={{ color: 'var(--text-muted)' }}>:L{caller.line_start || 1}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Copy Prompt Contract Button */}
      <button
        onClick={handleCopyAgentContract}
        style={{
          marginTop: 2,
          padding: '6px 10px',
          borderRadius: 6,
          backgroundColor: 'rgba(255, 255, 255, 0.06)',
          border: '1px solid rgba(255, 255, 255, 0.12)',
          color: copiedContract ? 'var(--accent-emerald)' : 'var(--text-primary)',
          fontSize: '0.7rem',
          fontWeight: 600,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 6,
          cursor: 'pointer',
          transition: 'all 0.15s ease',
        }}
      >
        {copiedContract ? <Check size={13} color="var(--accent-emerald)" /> : <Copy size={13} />}
        <span>{copiedContract ? 'Copied Contract to Clipboard!' : 'Copy Boundary Contract for Agent'}</span>
      </button>
    </div>
  );
};

export default BoundaryContractCard;
