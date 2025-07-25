<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'

const router = useRouter()
const route = useRoute()
const { isLoggedIn, logout: authLogout } = useAuth()

const logout = () => {
  authLogout()
  router.push('/')
}

// Check if we're in admin area
const isAdminRoute = computed(() => {
  return route.path.startsWith('/admin')
})

// Don't show main header in admin area
const showMainHeader = computed(() => {
  return !isAdminRoute.value
})
</script>

<template>
  <header v-if="showMainHeader">
    <h1>Demo Login App</h1>

    <nav>
      <RouterLink to="/">Home</RouterLink>
      <span v-if="!isLoggedIn">
        <RouterLink to="/login">Login</RouterLink>
        <RouterLink to="/register">Register</RouterLink>
      </span>
      <span v-if="isLoggedIn">
        <RouterLink to="/profile">Profile</RouterLink>
        <button @click="logout">Logout</button>
      </span>
      <span>
        <RouterLink to="/admin/login" class="admin-link">Admin Panel</RouterLink>
      </span>
    </nav>
  </header>

  <main :class="{ 'admin-main': isAdminRoute }">
    <RouterView />
  </main>
</template>

<style scoped>
header {
  padding: 1rem;
  border-bottom: 1px solid #ccc;
}

nav {
  margin-top: 1rem;
}

nav a, nav button {
  margin-right: 1rem;
  padding: 0.5rem 1rem;
  text-decoration: none;
  color: #007bff;
  border: none;
  background: none;
  cursor: pointer;
}

nav a:hover, nav button:hover {
  text-decoration: underline;
}

.admin-link {
  color: #dc3545 !important;
  font-weight: 600;
}

.admin-link:hover {
  color: #c82333 !important;
}

main {
  padding: 2rem;
}

.admin-main {
  padding: 0;
}
</style>
