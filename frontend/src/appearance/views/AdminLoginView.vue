<template>
  <div class="admin-login-container">
    <div class="login-card">
      <div class="login-header">
        <h1>Admin Login</h1>
        <p>Access the Rext Admin Panel</p>
      </div>

      <form @submit.prevent="handleLogin" class="login-form">
        <div class="form-group">
          <label for="email">Email</label>
          <input
            id="email"
            v-model="form.email"
            type="email"
            required
            placeholder="admin@localhost.com"
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="password">Password</label>
          <input
            id="password"
            v-model="form.password"
            type="password"
            required
            placeholder="Enter your password"
            :disabled="loading"
          />
        </div>

        <div v-if="error" class="error-message">
          {{ error }}
        </div>

        <button type="submit" class="login-btn" :disabled="loading">
          <span v-if="loading">Logging in...</span>
          <span v-else>Login</span>
        </button>
      </form>

      <div class="login-footer">
        <p>Default credentials: admin@localhost.com / admin</p>
        <router-link to="/" class="back-link">← Back to Main Site</router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { adminLoginHandler } from '@/bridge/client'

const router = useRouter()
const loading = ref(false)
const error = ref('')

const form = reactive({
  email: '',
  password: ''
})

const handleLogin = async () => {
  loading.value = true
  error.value = ''
  
  try {
    const response = await adminLoginHandler({
      body: {
        email: form.email,
        password: form.password
      }
    })
    
    if (response.data) {
      const data = response.data
      localStorage.setItem('adminToken', data.token)
      localStorage.setItem('adminEmail', data.email)
      router.push('/admin')
    } else if (response.error) {
      error.value = response.error.message || 'Login failed'
    }
  } catch (err: any) {
    console.error('Login error:', err)
    error.value = err.data?.message || 'Network error. Please try again.'
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.admin-login-container {
  height: 90vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 2rem;
}

.login-card {
  background: white;
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0,0,0,0.2);
  padding: 3rem;
  width: 100%;
  max-width: 400px;
}

.login-header {
  text-align: center;
  margin-bottom: 2rem;
}

.login-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.login-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.login-form {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  font-weight: 500;
  color: #333;
  font-size: 0.9rem;
}

.form-group input {
  padding: 0.75rem;
  border: 2px solid #e0e0e0;
  border-radius: 6px;
  font-size: 1rem;
  transition: border-color 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: #667eea;
}

.form-group input:disabled {
  background-color: #f5f5f5;
  cursor: not-allowed;
}

.error-message {
  background: #fee;
  color: #c33;
  padding: 0.75rem;
  border-radius: 6px;
  font-size: 0.9rem;
  border: 1px solid #fcc;
}

.login-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  padding: 1rem;
  border-radius: 6px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.login-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.login-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  transform: none;
}

.login-footer {
  margin-top: 2rem;
  text-align: center;
  padding-top: 1.5rem;
  border-top: 1px solid #e0e0e0;
}

.login-footer p {
  margin: 0 0 1rem 0;
  color: #666;
  font-size: 0.85rem;
}

.back-link {
  color: #667eea;
  text-decoration: none;
  font-size: 0.9rem;
  transition: color 0.2s;
}

.back-link:hover {
  color: #764ba2;
}

@media (max-width: 480px) {
  .login-card {
    padding: 2rem;
    margin: 1rem;
  }
  
  .admin-login-container {
    padding: 1rem;
  }
}
</style> 