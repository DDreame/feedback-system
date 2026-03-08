import { useState, FormEvent } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../hooks/useAuth';
import { login } from '../services/auth';

function Login() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [errors, setErrors] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const navigate = useNavigate();
  const { login: storeLogin } = useAuth();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setErrors([]);

    // Validate inputs
    const validationErrors: string[] = [];
    if (!email.trim()) {
      validationErrors.push('Email is required');
    }
    if (!password.trim()) {
      validationErrors.push('Password is required');
    }

    if (validationErrors.length > 0) {
      setErrors(validationErrors);
      return;
    }

    setIsLoading(true);

    try {
      const result = await login(email, password);
      storeLogin(result);
      navigate('/');
    } catch (err) {
      setErrors([err instanceof Error ? err.message : 'Login failed']);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{
      display: 'flex',
      justifyContent: 'center',
      alignItems: 'center',
      minHeight: '100vh',
      backgroundColor: '#f5f5f5',
    }}>
      <div style={{
        backgroundColor: 'white',
        padding: '48px',
        borderRadius: '8px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.1)',
        width: '400px',
      }}>
        <h1 style={{ marginBottom: '24px', textAlign: 'center' }}>Login</h1>
        <form onSubmit={handleSubmit}>
          <div style={{ marginBottom: '16px' }}>
            <label htmlFor="email" style={{ display: 'block', marginBottom: '8px' }}>Email</label>
            <input
              id="email"
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              disabled={isLoading}
              style={{
                width: '100%',
                padding: '12px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                boxSizing: 'border-box',
              }}
            />
          </div>
          <div style={{ marginBottom: '24px' }}>
            <label htmlFor="password" style={{ display: 'block', marginBottom: '8px' }}>Password</label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              disabled={isLoading}
              style={{
                width: '100%',
                padding: '12px',
                border: '1px solid #ddd',
                borderRadius: '4px',
                boxSizing: 'border-box',
              }}
            />
          </div>
          {errors.length > 0 && (
            <div style={{
              marginBottom: '16px',
              padding: '12px',
              backgroundColor: '#fee2e2',
              border: '1px solid #fecaca',
              borderRadius: '4px',
              color: '#dc2626',
              fontSize: '14px',
            }}>
              {errors.map((err, index) => (
                <div key={index}>{err}</div>
              ))}
            </div>
          )}
          <button
            type="submit"
            disabled={isLoading}
            style={{
              width: '100%',
              padding: '12px',
              backgroundColor: '#1a1a2e',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              opacity: isLoading ? 0.7 : 1,
            }}
          >
            {isLoading ? 'Signing in...' : 'Sign In'}
          </button>
        </form>
      </div>
    </div>
  );
}

export default Login;
