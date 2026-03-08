import { useEffect, useState } from 'react';

interface HealthStatus {
  backend: 'checking' | 'healthy' | 'unhealthy';
  database: 'checking' | 'healthy' | 'unhealthy';
  redis: 'checking' | 'healthy' | 'unhealthy';
  websocket: 'checking' | 'healthy' | 'unhealthy';
  lastChecked: string | null;
  error?: string;
}

function Status() {
  const [status, setStatus] = useState<HealthStatus>({
    backend: 'checking',
    database: 'checking',
    redis: 'checking',
    websocket: 'checking',
    lastChecked: null,
  });
  const [isRefreshing, setIsRefreshing] = useState(false);

  const apiUrl = import.meta.env.VITE_API_URL || 'http://localhost:3000';

  const checkHealth = async () => {
    setIsRefreshing(true);

    const newStatus: HealthStatus = {
      backend: 'checking',
      database: 'checking',
      redis: 'checking',
      websocket: 'checking',
      lastChecked: new Date().toISOString(),
    };

    try {
      // Check backend health
      const response = await fetch(`${apiUrl}/health`, {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' },
        signal: AbortSignal.timeout(5000),
      });

      if (response.ok) {
        const data = await response.json();
        newStatus.backend = 'healthy';

        // Check database status from health response
        if (data.database === 'connected') {
          newStatus.database = 'healthy';
        } else if (data.database === 'disconnected') {
          newStatus.database = 'unhealthy';
        }

        // Check Redis status from health response
        if (data.redis === 'connected') {
          newStatus.redis = 'healthy';
        } else if (data.redis === 'disconnected') {
          newStatus.redis = 'unhealthy';
        }
      } else {
        newStatus.backend = 'unhealthy';
        newStatus.error = `Backend returned status ${response.status}`;
      }
    } catch (err: any) {
      newStatus.backend = 'unhealthy';
      newStatus.error = err.message || 'Failed to connect to backend';
    }

    // WebSocket status - assume healthy if backend is healthy
    newStatus.websocket = newStatus.backend === 'healthy' ? 'healthy' : 'unhealthy';

    setStatus(newStatus);
    setIsRefreshing(false);
  };

  useEffect(() => {
    checkHealth();
  }, [apiUrl]);

  const getStatusColor = (s: 'checking' | 'healthy' | 'unhealthy') => {
    switch (s) {
      case 'healthy':
        return '#28a745';
      case 'unhealthy':
        return '#dc3545';
      case 'checking':
        return '#ffc107';
      default:
        return '#6c757d';
    }
  };

  const getStatusText = (s: 'checking' | 'healthy' | 'unhealthy') => {
    switch (s) {
      case 'healthy':
        return 'Healthy';
      case 'unhealthy':
        return 'Unhealthy';
      case 'checking':
        return 'Checking...';
      default:
        return 'Unknown';
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, -apple-system, sans-serif', maxWidth: '800px', margin: '0 auto' }}>
      <h1>System Status</h1>

      {/* Navigation */}
      <nav style={{ marginBottom: '20px', padding: '10px', background: '#f5f5f5', borderRadius: '4px' }}>
        <a href="/" style={{ marginRight: '20px', textDecoration: 'none', color: '#0066cc' }}>SDK Simulator</a>
        <a href="/status" style={{ textDecoration: 'none', color: '#0066cc' }}>System Status</a>
      </nav>

      {/* Refresh Button */}
      <div style={{ marginBottom: '20px' }}>
        <button
          onClick={checkHealth}
          disabled={isRefreshing}
          style={{
            padding: '8px 16px',
            background: isRefreshing ? '#ccc' : '#007bff',
            color: 'white',
            border: 'none',
            borderRadius: '4px',
            cursor: isRefreshing ? 'not-allowed' : 'pointer',
          }}
        >
          {isRefreshing ? 'Refreshing...' : 'Refresh'}
        </button>
      </div>

      {/* Error Display */}
      {status.error && (
        <div style={{ marginBottom: '20px', padding: '10px', background: '#f8d7da', color: '#721c24', borderRadius: '4px' }}>
          <strong>Error:</strong> {status.error}
        </div>
      )}

      {/* Status Cards */}
      <div style={{ display: 'grid', gap: '15px' }}>
        {/* Backend Status */}
        <div style={{ padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0, marginBottom: '10px' }}>Backend API</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                borderRadius: '50%',
                backgroundColor: getStatusColor(status.backend),
              }}
            />
            <span style={{ fontSize: '18px' }}>{getStatusText(status.backend)}</span>
          </div>
          <div style={{ marginTop: '10px', fontSize: '14px', color: '#666' }}>
            URL: <code>{apiUrl}</code>
          </div>
        </div>

        {/* Database Status */}
        <div style={{ padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0, marginBottom: '10px' }}>Database (PostgreSQL)</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                borderRadius: '50%',
                backgroundColor: getStatusColor(status.database),
              }}
            />
            <span style={{ fontSize: '18px' }}>{getStatusText(status.database)}</span>
          </div>
        </div>

        {/* Redis Status */}
        <div style={{ padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0, marginBottom: '10px' }}>Cache (Redis)</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                borderRadius: '50%',
                backgroundColor: getStatusColor(status.redis),
              }}
            />
            <span style={{ fontSize: '18px' }}>{getStatusText(status.redis)}</span>
          </div>
        </div>

        {/* WebSocket Status */}
        <div style={{ padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
          <h2 style={{ marginTop: 0, marginBottom: '10px' }}>WebSocket</h2>
          <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
            <div
              style={{
                width: '12px',
                height: '12px',
                borderRadius: '50%',
                backgroundColor: getStatusColor(status.websocket),
              }}
            />
            <span style={{ fontSize: '18px' }}>{getStatusText(status.websocket)}</span>
          </div>
        </div>
      </div>

      {/* Last Checked */}
      {status.lastChecked && (
        <div style={{ marginTop: '20px', fontSize: '14px', color: '#666' }}>
          Last checked: {new Date(status.lastChecked).toLocaleString()}
        </div>
      )}
    </div>
  );
}

export default Status;
