<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
import { ref, onMounted } from 'vue'

const isLoggedIn = ref(false)

onMounted(() => {
  // Check if user has a token
  isLoggedIn.value = !!localStorage.getItem('token')
})

const logout = () => {
  localStorage.removeItem('token')
  isLoggedIn.value = false
  window.location.href = '/'
}
</script>

<template>
  <header>
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
    </nav>
  </header>

  <main>
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

main {
  padding: 2rem;
}
</style>
