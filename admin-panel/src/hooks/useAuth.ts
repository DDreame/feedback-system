import { useAuthStore, User } from '../stores/authStore';

interface LoginParams {
  user: User;
  accessToken: string;
  refreshToken: string;
}

export const useAuth = () => {
  const {
    isAuthenticated,
    user,
    accessToken,
    refreshToken,
    login: storeLogin,
    logout: storeLogout,
    setUser: storeSetUser,
    setTokens: storeSetTokens,
  } = useAuthStore();

  const login = (params: LoginParams) => {
    storeLogin(params);
  };

  const logout = () => {
    storeLogout();
  };

  const setUser = (user: User) => {
    storeSetUser(user);
  };

  const setTokens = (accessToken: string, refreshToken: string) => {
    storeSetTokens(accessToken, refreshToken);
  };

  return {
    isAuthenticated,
    user,
    accessToken,
    refreshToken,
    login,
    logout,
    setUser,
    setTokens,
  };
};
