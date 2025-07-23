<template>
  <div>
    <h2>Login</h2>

    <form @submit.prevent="handleLogin">
      <div>
        <label for="email">Email:</label>
        <br>
        <input type="email" id="email" v-model="email" required>
      </div>

      <div>
        <label for="password">Password:</label>
        <br>
        <input type="password" id="password" v-model="password" required>
      </div>

      <div>
        <button type="submit" :disabled="loading">
          {{ loading ? 'Logging in...' : 'Login' }}
        </button>
      </div>
    </form>

    <div v-if="error" style="color: red; margin-top: 1rem;">
      {{ error }}
    </div>

    <p>
      Don't have an account? <router-link to="/register">Register here</router-link>
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

const handleLogin = async () => {
  loading.value = true
  error.value = ''

  try {
    const response = await fetch('http://127.0.0.1:3000/api/v1/auth/login', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        email: email.value,
        password: password.value,
      }),
    })

    const data = await response.json()

    if (response.ok) {
      localStorage.setItem('token', data.token)
      router.push('/profile')
    } else {
      error.value = data.message || 'Login failed'
    }
  } catch (err) {
    error.value = 'Network error. Please make sure the backend server is running.'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
form div {
  margin-bottom: 1rem;
}

input {
  padding: 0.5rem;
  font-size: 1rem;
  width: 250px;
}

button {
  padding: 0.5rem 1rem;
  font-size: 1rem;
  background-color: #007bff;
  color: white;
  border: none;
  cursor: pointer;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

button:hover:not(:disabled) {
  background-color: #0056b3;
}
</style>