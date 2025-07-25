<template>
  <div>
    <h2>Register</h2>

    <form @submit.prevent="handleRegister">
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
          autocomplete="email">
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
          autocomplete="new-password">
        <div v-if="passwordError" class="error-message">{{ passwordError }}</div>
      </div>

      <div>
        <button type="submit" :disabled="loading || hasValidationErrors">
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
const router = useRouter()

const email = ref('')
const password = ref('')
const loading = ref(false)
const error = ref('')
const success = ref('')
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
    zRegisterRequest.shape.email.parse(email.value)
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
    zRegisterRequest.shape.password.parse(password.value)
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
    zRegisterRequest.parse({
      email: email.value,
      password: password.value
    })
    return true
  } catch (err) {
    return false
  }
}

const handleRegister = async () => {
  error.value = ''
  success.value = ''
  
  // validate form before submitting
  if (!validateForm()) {
    return
  }

  loading.value = true

  try {
    const registerData: RegisterRequest = {
      email: email.value,
      password: password.value,
    }

    const response = await registerHandler({
      body: registerData,
    })

    // Check if there's an error in the response
    if (response.error) {
      if (response.error.message) {
        error.value = response.error.message
      } else {
        error.value = 'Registration failed. Please try again.'
      }
      return
    }

    // Check if we have successful data
    if (response.data) {
      success.value = response.data.message + ' Redirecting to login...'
      setTimeout(() => {
        router.push('/login')
      }, 2000)
    } else {
      error.value = 'Registration failed. Please try again.'
    }
  } catch (err: any) {
    console.error(err)
    // This catch block will only trigger for network errors or other exceptions
    if (err.error?.message) {
      error.value = err.error.message
    } else {
      error.value = 'Unknown error. Please try again.'
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