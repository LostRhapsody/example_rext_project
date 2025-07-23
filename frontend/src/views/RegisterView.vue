<template>
  <div>
    <h2>Register</h2>

    <form @submit.prevent="handleRegister">
      <div>
        <label for="email">Email:</label>
        <br>
        <input type="email" id="email" v-model="email" required>
      </div>

      <div>
        <label for="password">Password:</label>
        <br>
        <input type="password" id="password" v-model="password" required minlength="6">
        <small>Password must be at least 6 characters</small>
      </div>

      <div>
        <button type="submit" :disabled="loading">
          {{ loading ? 'Creating account...' : 'Register' }}
        </button>
      </div>
    </form>

    <div v-if="error" style="color: red; margin-top: 1rem;">
      {{ error }}
    </div>

    <div v-if="success" style="color: green; margin-top: 1rem;">
      {{ success }}
    </div>

    <p>
      Already have an account? <router-link to="/login">Login here</router-link>
    </p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { registerHandler, type RegisterRequest } from '../client'

const router = useRouter()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const success = ref('')

const handleRegister = async () => {
  loading.value = true
  error.value = ''
  success.value = ''

  try {
    const registerData: RegisterRequest = {
      email: email.value,
      password: password.value,
    }

    const response = await registerHandler({
      body: registerData,
    })
    
    if (response.data) {
      success.value = response.data.message + ' Redirecting to login...'
      setTimeout(() => {
        router.push('/login')
      }, 2000)
    }
  } catch (err: any) {
    if (err.error?.message) {
      error.value = err.error.message
    } else {
      error.value = 'Network error. Please make sure the backend server is running.'
    }
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

small {
  display: block;
  color: #666;
  margin-top: 0.25rem;
}

button {
  padding: 0.5rem 1rem;
  font-size: 1rem;
  background-color: #28a745;
  color: white;
  border: none;
  cursor: pointer;
}

button:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

button:hover:not(:disabled) {
  background-color: #218838;
}
</style>