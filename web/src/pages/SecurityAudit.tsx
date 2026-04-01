import { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  getAudit,
  getServer,
  getServerAudits,
  startAudit,
  type SecurityAudit,
  type Server,
} from '../lib/api';

export default function SecurityAuditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [server, setServer] = useState<Server | null>(null);
  const [audits, setAudits] = useState<SecurityAudit[]>([]);
  const [activeAudit, setActiveAudit] = useState<SecurityAudit | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const loadData = async () => {
    if (!id) return;
    try {
      const [s, a] = await Promise.all([getServer(id), getServerAudits(id)]);
      setServer(s);
      setAudits(a);
      if (a.length > 0) {
        setActiveAudit(a[0]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load data');
    }
  };

  useEffect(() => {
    loadData();
  }, [id]);

  // Poll running audits
  useEffect(() => {
    if (!activeAudit || activeAudit.status !== 'running') return;
    const interval = setInterval(async () => {
      try {
        const updated = await getAudit(activeAudit.id);
        setActiveAudit(updated);
        if (updated.status !== 'running') {
          clearInterval(interval);
          loadData();
        }
      } catch {
        clearInterval(interval);
      }
    }, 2000);
    return () => clearInterval(interval);
  }, [activeAudit?.id, activeAudit?.status]);

  const handleStartAudit = async () => {
    if (!id) return;
    setLoading(true);
    setError('');
    try {
      const audit = await startAudit(id);
      setActiveAudit(audit);
      setAudits((prev) => [audit, ...prev]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start audit');
    } finally {
      setLoading(false);
    }
  };

  const scoreColor = (score: number | null) => {
    if (score === null) return 'text-gray-400';
    if (score >= 80) return 'text-green-400';
    if (score >= 60) return 'text-yellow-400';
    return 'text-red-400';
  };

  const scoreBg = (score: number | null) => {
    if (score === null) return 'bg-gray-600/20';
    if (score >= 80) return 'bg-green-600/20';
    if (score >= 60) return 'bg-yellow-600/20';
    return 'bg-red-600/20';
  };

  const statusIcon = (status: string) => {
    if (status === 'pass') return '✓';
    if (status === 'warn') return '⚠';
    return '✗';
  };

  const statusColor = (status: string) => {
    if (status === 'pass') return 'text-green-400';
    if (status === 'warn') return 'text-yellow-400';
    return 'text-red-400';
  };

  const statusBg = (status: string) => {
    if (status === 'pass') return 'bg-green-600/20';
    if (status === 'warn') return 'bg-yellow-600/20';
    return 'bg-red-600/20';
  };

  const categoryLabel = (cat: string) => {
    const labels: Record<string, string> = {
      ssh: 'SSH',
      network: 'Network',
      system: 'System',
      auth: 'Authentication',
      filesystem: 'Filesystem',
      connectivity: 'Connectivity',
    };
    return labels[cat] || cat;
  };

  return (
    <div>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-4">
          <button
            onClick={() => navigate('/servers')}
            className="text-gray-400 hover:text-white transition-colors"
          >
            ← Back
          </button>
          <div>
            <h1 className="text-2xl font-bold text-white">Security Audit</h1>
            {server && (
              <p className="text-sm text-gray-400 mt-1">
                {server.name} ({server.host})
              </p>
            )}
          </div>
        </div>
        <button
          onClick={handleStartAudit}
          disabled={loading || activeAudit?.status === 'running'}
          className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading || activeAudit?.status === 'running'
            ? 'Scanning...'
            : 'Run Security Scan'}
        </button>
      </div>

      {error && (
        <div className="bg-red-600/20 border border-red-600/30 text-red-400 text-sm p-3 rounded-lg mb-6">
          {error}
        </div>
      )}

      {/* Active Audit Score */}
      {activeAudit && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-6 mb-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-lg font-semibold text-white">
              {activeAudit.status === 'running' ? 'Scanning...' : 'Latest Result'}
            </h2>
            <div className="text-xs text-gray-500">
              {activeAudit.created_at &&
                new Date(activeAudit.created_at).toLocaleString()}
            </div>
          </div>

          {activeAudit.status === 'running' ? (
            <div className="flex items-center gap-3">
              <div className="animate-spin h-8 w-8 border-2 border-forge-500 border-t-transparent rounded-full" />
              <span className="text-gray-400">
                Running security checks on {server?.host}...
              </span>
            </div>
          ) : (
            <>
              {/* Score Circle */}
              <div className="flex items-center gap-8 mb-6">
                <div
                  className={`w-24 h-24 rounded-full ${scoreBg(activeAudit.score)} flex items-center justify-center border-2 ${
                    activeAudit.score !== null && activeAudit.score >= 80
                      ? 'border-green-600/40'
                      : activeAudit.score !== null && activeAudit.score >= 60
                        ? 'border-yellow-600/40'
                        : 'border-red-600/40'
                  }`}
                >
                  <span
                    className={`text-3xl font-bold ${scoreColor(activeAudit.score)}`}
                  >
                    {activeAudit.score ?? '—'}
                  </span>
                </div>
                <div>
                  <p className={`text-lg font-semibold ${scoreColor(activeAudit.score)}`}>
                    {activeAudit.score !== null && activeAudit.score >= 80
                      ? 'Good'
                      : activeAudit.score !== null && activeAudit.score >= 60
                        ? 'Fair'
                        : 'Needs Attention'}
                  </p>
                  <p className="text-sm text-gray-400 mt-1">
                    {activeAudit.results?.filter((r) => r.status === 'pass').length ?? 0} passed,{' '}
                    {activeAudit.results?.filter((r) => r.status === 'warn').length ?? 0} warnings,{' '}
                    {activeAudit.results?.filter((r) => r.status === 'fail').length ?? 0} failed
                  </p>
                </div>
              </div>

              {/* Check Results */}
              <div className="space-y-3">
                {activeAudit.results?.map((check, i) => (
                  <div
                    key={i}
                    className="bg-gray-800/50 border border-gray-700/50 rounded-lg p-4 flex items-center justify-between"
                  >
                    <div className="flex items-center gap-3">
                      <span
                        className={`w-8 h-8 rounded-full ${statusBg(check.status)} flex items-center justify-center text-sm ${statusColor(check.status)}`}
                      >
                        {statusIcon(check.status)}
                      </span>
                      <div>
                        <p className="text-sm font-medium text-white">
                          {check.name}
                        </p>
                        <p className="text-xs text-gray-400">{check.detail}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="px-2 py-0.5 bg-forge-600/20 text-forge-400 text-xs rounded-md">
                        {categoryLabel(check.category)}
                      </span>
                      <span className={`text-sm font-mono ${statusColor(check.status)}`}>
                        {check.points}/{check.max_points}
                      </span>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      )}

      {/* Audit History */}
      {audits.length > 1 && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-5">
          <h2 className="text-lg font-semibold text-white mb-4">
            Audit History
          </h2>
          <div className="space-y-2">
            {audits.map((audit) => (
              <button
                key={audit.id}
                onClick={() => setActiveAudit(audit)}
                className={`w-full text-left px-4 py-3 rounded-lg border transition-colors flex items-center justify-between ${
                  activeAudit?.id === audit.id
                    ? 'bg-gray-800 border-forge-600/50'
                    : 'bg-gray-800/30 border-gray-700/50 hover:border-gray-700'
                }`}
              >
                <div className="flex items-center gap-3">
                  <span
                    className={`text-sm font-bold ${scoreColor(audit.score)}`}
                  >
                    {audit.status === 'running'
                      ? '...'
                      : audit.score !== null
                        ? `${audit.score}%`
                        : '—'}
                  </span>
                  <span className="text-sm text-gray-400">
                    {audit.created_at
                      ? new Date(audit.created_at).toLocaleString()
                      : '—'}
                  </span>
                </div>
                <span className="text-xs text-gray-500">
                  by {audit.created_by || 'system'}
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Empty State */}
      {audits.length === 0 && !error && (
        <div className="bg-gray-900 border border-gray-800 rounded-xl p-12 text-center">
          <div className="text-4xl mb-4">🛡️</div>
          <h3 className="text-lg font-semibold text-white mb-2">
            No Security Audits Yet
          </h3>
          <p className="text-sm text-gray-400 mb-4">
            Run a security scan to check SSH configuration, firewall status,
            open ports, and more.
          </p>
          <button
            onClick={handleStartAudit}
            disabled={loading}
            className="px-4 py-2 bg-forge-600 text-white text-sm font-medium rounded-lg hover:bg-forge-700 transition-colors disabled:opacity-50"
          >
            Run First Scan
          </button>
        </div>
      )}
    </div>
  );
}
