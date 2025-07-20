<template>
  <div>
    <h2>User Profile</h2>

    <div v-if="loading">
      Loading profile...
    </div>

    <div v-else-if="error" style="color: red;">
      {{ error }}
    </div>

    <div v-else-if="profile">
      <p><strong>User ID:</strong> {{ profile.id }}</p>
      <p><strong>Email:</strong> {{ profile.email }}</p>
      <p><strong>Created At:</strong> {{ formatDate(profile.created_at) }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

interface Profile {
  id: string
  email: string
  created_at?: string
}

const profile = ref<Profile | null>(null)
const loading = ref(true)
const error = ref('')

const formatDate = (dateString?: string) => {
  if (!dateString) return 'N/A'
  return new Date(dateString).toLocaleString()
}

const fetchProfile = async () => {
  const token = localStorage.getItem('token')

  if (!token) {
    router.push('/login')
    return
  }

  try {
    const response = await fetch('http://127.0.0.1:3000/profile', {
      headers: {
        'Authorization': `Bearer ${token}`,
      },
    })

    const data = await response.json()

    if (response.ok) {
      profile.value = data
    } else {
      if (response.status === 401) {
        localStorage.removeItem('token')
        router.push('/login')
      } else {
        error.value = data.message || 'Failed to fetch profile'
      }
    }
  } catch (err) {
    error.value = 'Network error. Please make sure the backend server is running.'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchProfile()
})
</script>