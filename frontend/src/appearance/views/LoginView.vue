<template>
  <div>
    <h2>Login</h2>

    <form @submit.prevent="handleLogin">
      <div>
        <label for="email">Email:</label>
        <br>
        <input 
          type="email" 
          id="email" 
          v-model="email" 
          :class="{ 'error': emailError }"
          @blur="validateEmail"
          required
          autocomplete="email"
        >
        <div v-if="emailError" class="error-message">{{ emailError }}</div>
      </div>

      <div>
        <label for="password">Password:</label>
        <br>
        <input 
          type="password" 
          id="password" 
          v-model="password" 
          :class="{ 'error': passwordError }"
          @blur="validatePassword"
          required
          autocomplete="current-password"
        >
        <div v-if="passwordError" class="error-message">{{ passwordError }}</div>
      </div>

      <div>
        <button type="submit" :disabled="loading || hasValidationErrors">
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
const { login } = useAuth()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const emailError = ref('')
const passwordError = ref('')

// Computed property to check if there are any validation errors
const hasValidationErrors = computed(() => {
  if (emailError.value !== '' || passwordError.value !== '') {
    return true
  }
  return false
})

// Validate email using Zod schema
const validateEmail = () => {
  try {
    zLoginRequest.shape.email.parse(email.value)
    emailError.value = ''
  } catch (err) {
    if (err instanceof Error) {
      emailError.value = err.message
    } else {
      emailError.value = 'Invalid email format'
    }
  }
}

// Validate password using Zod schema
const validatePassword = () => {
  try {
    zLoginRequest.shape.password.parse(password.value)
    passwordError.value = ''
  } catch (err) {
    if (err instanceof Error) {
      passwordError.value = err.message
    } else {
      passwordError.value = 'Password is required'
    }
  }
}

// Validate entire form using Zod schema
const validateForm = () => {
  validateEmail()
  validatePassword()
  
  try {
    zLoginRequest.parse({
      email: email.value,
      password: password.value
    })
    return true
  } catch (err) {
    return false
  }
}

const handleLogin = async () => {
  // Clear previous errors
  error.value = ''
  
  // Validate form before submission
  if (!validateForm()) {
    return
  }

  loading.value = true

  try {
    const loginData: LoginRequest = {
      email: email.value,
      password: password.value,
    }

    const response = await loginHandler({
      body: loginData,
    })

    if (response.data) {
      login(response.data.token)
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

input.error {
  border-color: #dc3545;
  border-width: 2px;
}

.error-message {
  color: #dc3545;
  font-size: 0.875rem;
  margin-top: 0.25rem;
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