import { FeedbackWidget, Message } from '@feedback-system/web-sdk';
import { useEffect, useRef, useState } from 'react';
import { Routes, Route, Link } from 'react-router-dom';
import Status from './pages/Status';

function SDKSimulator() {
  const [apiKey, setApiKey] = useState<string>(import.meta.env.VITE_API_KEY || '');
  const [projectId, setProjectId] = useState<string>(import.meta.env.VITE_PROJECT_ID || '');
  const [deviceId, setDeviceId] = useState<string>('');
  const [apiUrl] = useState<string>(import.meta.env.VITE_API_URL || 'http://localhost:3000');
  const [status, setStatus] = useState<string>('Not connected');
  const [connected, setConnected] = useState<boolean>(false);
  const [messages, setMessages] = useState<Message[]>([]);
  const [newMessage, setNewMessage] = useState<string>('');
  const [error, setError] = useState<string>('');

  const widgetRef = useRef<FeedbackWidget | null>(null);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (widgetRef.current) {
        widgetRef.current.destroy();
      }
    };
  }, []);

  const handleConnect = async () => {
    setError('');
    setMessages([]);

    // Cleanup existing widget
    if (widgetRef.current) {
      widgetRef.current.destroy();
    }

    if (!apiKey || !projectId) {
      setError('API Key and Project ID are required');
      return;
    }

    setStatus('Initializing...');

    try {
      const newWidget = new FeedbackWidget({
        apiKey,
        projectId,
        apiUrl,
        debug: true,
        onMessage: (msg: Message) => {
          console.log('New message:', msg);
          setMessages((prev) => [...prev, msg]);
        },
        onConnectionChange: (isConnected: boolean) => {
          console.log('Connection changed:', isConnected);
          setConnected(isConnected);
          setStatus(isConnected ? 'Connected' : 'Disconnected');
        },
      });

      // Set device ID if provided
      if (deviceId) {
        localStorage.setItem('feedback_device_id', deviceId);
      }

      await newWidget.init();

      widgetRef.current = newWidget;

      // Get initial user and conversation info
      const endUser = newWidget.getEndUser();

      if (endUser) {
        setDeviceId(endUser.deviceId);
      }

      console.log('Initialized:', { endUser, conversation: newWidget.getConversation() });
    } catch (err: any) {
      console.error('Init error:', err);
      setStatus('Error');
      setError(err.message || 'Failed to initialize SDK');
    }
  };

  const handleDisconnect = () => {
    if (widgetRef.current) {
      widgetRef.current.destroy();
      widgetRef.current = null;
      setConnected(false);
      setStatus('Disconnected');
      setMessages([]);
    }
  };

  const handleSendMessage = async () => {
    if (!newMessage.trim() || !widgetRef.current) return;

    try {
      const message = await widgetRef.current.sendMessage(newMessage);
      setMessages((prev) => [...prev, message]);
      setNewMessage('');
    } catch (err: any) {
      console.error('Send error:', err);
      setError(err.message || 'Failed to send message');
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, -apple-system, sans-serif', maxWidth: '800px', margin: '0 auto' }}>
      <h1>Feedback System SDK Simulator</h1>

      {/* Navigation */}
      <nav style={{ marginBottom: '20px', padding: '10px', background: '#f5f5f5', borderRadius: '4px' }}>
        <Link to="/" style={{ marginRight: '20px', textDecoration: 'none', color: '#0066cc' }}>SDK Simulator</Link>
        <Link to="/status" style={{ textDecoration: 'none', color: '#0066cc' }}>System Status</Link>
      </nav>

      {/* Configuration Panel */}
      <div style={{ marginBottom: '20px', padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
        <h2 style={{ marginTop: 0 }}>Configuration</h2>

        <div style={{ marginBottom: '10px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
            API Key:
            <input
              type="text"
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              placeholder="proj_xxxxx"
              style={{ marginLeft: '10px', padding: '5px', width: '300px', fontFamily: 'monospace' }}
            />
          </label>
        </div>

        <div style={{ marginBottom: '10px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
            Project ID:
            <input
              type="text"
              value={projectId}
              onChange={(e) => setProjectId(e.target.value)}
              placeholder="uuid"
              style={{ marginLeft: '10px', padding: '5px', width: '300px', fontFamily: 'monospace' }}
            />
          </label>
        </div>

        <div style={{ marginBottom: '10px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
            Device ID (optional):
            <input
              type="text"
              value={deviceId}
              onChange={(e) => setDeviceId(e.target.value)}
              placeholder="Leave empty to auto-generate"
              style={{ marginLeft: '10px', padding: '5px', width: '300px', fontFamily: 'monospace' }}
            />
          </label>
        </div>

        <div style={{ marginBottom: '10px' }}>
          <label style={{ display: 'block', marginBottom: '5px', fontWeight: 'bold' }}>
            API URL:
            <span style={{ marginLeft: '10px', fontFamily: 'monospace' }}>{apiUrl}</span>
          </label>
        </div>

        <div>
          {connected ? (
            <button
              onClick={handleDisconnect}
              style={{
                padding: '8px 16px',
                background: '#dc3545',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Disconnect
            </button>
          ) : (
            <button
              onClick={handleConnect}
              style={{
                padding: '8px 16px',
                background: '#28a745',
                color: 'white',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Connect
            </button>
          )}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div style={{ marginBottom: '20px', padding: '10px', background: '#f8d7da', color: '#721c24', borderRadius: '4px' }}>
          <strong>Error:</strong> {error}
        </div>
      )}

      {/* Status Panel */}
      <div style={{ marginBottom: '20px', padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
        <h2 style={{ marginTop: 0 }}>Connection Status</h2>
        <div style={{ marginBottom: '10px' }}>
          <strong>Status:</strong>{' '}
          <span style={{ color: connected ? '#28a745' : '#dc3545' }}>{status}</span>
        </div>
        <div>
          <strong>Connected:</strong> {connected ? 'Yes' : 'No'}
        </div>
      </div>

      {/* Messages Panel */}
      <div style={{ padding: '15px', border: '1px solid #ddd', borderRadius: '8px' }}>
        <h2 style={{ marginTop: 0 }}>Messages</h2>

        <div
          style={{
            height: '300px',
            overflowY: 'auto',
            border: '1px solid #ddd',
            borderRadius: '4px',
            padding: '10px',
            marginBottom: '10px',
            background: '#fafafa',
          }}
        >
          {messages.length === 0 ? (
            <div style={{ color: '#666', fontStyle: 'italic' }}>No messages yet</div>
          ) : (
            messages.map((msg, index) => (
              <div
                key={msg.id || index}
                style={{
                  marginBottom: '10px',
                  padding: '8px',
                  borderRadius: '4px',
                  background: msg.senderType === 'end_user' ? '#e3f2fd' : '#f5f5f5',
                  borderLeft: msg.senderType === 'end_user' ? '3px solid #2196f3' : '3px solid #4caf50',
                }}
              >
                <div style={{ fontSize: '12px', color: '#666', marginBottom: '4px' }}>
                  <strong>{msg.senderType}</strong> - {new Date(msg.createdAt).toLocaleTimeString()}
                </div>
                <div>{msg.content}</div>
              </div>
            ))
          )}
        </div>

        <div style={{ display: 'flex', gap: '10px' }}>
          <input
            type="text"
            value={newMessage}
            onChange={(e) => setNewMessage(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Type a message..."
            disabled={!connected}
            style={{
              flex: 1,
              padding: '8px',
              border: '1px solid #ddd',
              borderRadius: '4px',
            }}
          />
          <button
            onClick={handleSendMessage}
            disabled={!connected || !newMessage.trim()}
            style={{
              padding: '8px 16px',
              background: connected ? '#007bff' : '#ccc',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: connected ? 'pointer' : 'not-allowed',
            }}
          >
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

function App() {
  return (
    <Routes>
      <Route path="/" element={<SDKSimulator />} />
      <Route path="/status" element={<Status />} />
    </Routes>
  );
}

export default App;
