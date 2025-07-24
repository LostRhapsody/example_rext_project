<template>
  <div>
    <h2>Welcome to the Demo Login App</h2>
    <p>This is a simple demonstration of authentication using Vue 3 and a Rust backend.</p>

    <div v-if="!isLoggedIn">
      <p>Please <router-link to="/login">login</router-link> or <router-link to="/register">register</router-link> to access your profile.</p>
    </div>

    <div v-if="isLoggedIn">
      <p>You are logged in! <router-link to="/profile">View your profile</router-link></p>
    </div>
  </div>
</template>

<script setup lang="ts">
const { isLoggedIn } = useAuth()
</script>