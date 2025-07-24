export function useAuth() {
  const route = useRoute()
  const isLoggedIn = ref(false)

  const checkAuthState = () => {
    isLoggedIn.value = !!localStorage.getItem('token')
  }

  const logout = () => {
    localStorage.removeItem('token')
    isLoggedIn.value = false
  }

  const login = (token: string) => {
    localStorage.setItem('token', token)
    isLoggedIn.value = true
  }

  onMounted(() => {
    checkAuthState()
  })

  // Watch for route changes and update auth state
  watch(route, () => {
    checkAuthState()
  })

  return {
    isLoggedIn,
    checkAuthState,
    logout,
    login
  }
}