import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from "react"
import { api, type User } from "./api"

interface AuthContextValue {
  user: User | null
  loading: boolean
  authEnabled: boolean
  logout: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue>({
  user: null,
  loading: true,
  authEnabled: false,
  logout: async () => {},
})

// eslint-disable-next-line react-refresh/only-export-components
export function useAuth() {
  return useContext(AuthContext)
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null)
  const [loading, setLoading] = useState(true)
  const [authEnabled, setAuthEnabled] = useState(false)

  useEffect(() => {
    api
      .getMe()
      .then((data) => {
        setUser(data.user)
        setAuthEnabled(data.auth_enabled)
      })
      .catch(() => {
        // 401 or network error — user is not authenticated
        setUser(null)
        setAuthEnabled(true) // Assume auth is enabled if /me fails with 401
      })
      .finally(() => setLoading(false))
  }, [])

  const logout = useCallback(async () => {
    try {
      await api.logout()
    } finally {
      setUser(null)
      window.location.href = "/login"
    }
  }, [])

  return (
    <AuthContext.Provider value={{ user, loading, authEnabled, logout }}>
      {children}
    </AuthContext.Provider>
  )
}
