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

const router = useRouter()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')

const handleLogin = async () => {
  loading.value = true
  error.value = ''

  try {
    const loginData: LoginRequest = {
      email: email.value,
      password: password.value,
    }

    const response = await loginHandler({
      body: loginData,
    })

    if (response.data) {
      localStorage.setItem('token', response.data.token)
      router.push('/profile')
    } else {
      error.value = 'Login failed'
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