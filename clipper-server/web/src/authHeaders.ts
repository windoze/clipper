const AUTH_TOKEN_KEY = "clipper-web-token";

export function getAuthHeaders(): HeadersInit {
  const token = localStorage.getItem(AUTH_TOKEN_KEY);
  if (!token) {
    return {};
  }

  return {
    Authorization: `Bearer ${token}`,
  };
}
