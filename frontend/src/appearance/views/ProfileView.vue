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
import { profileHandler, type ProfileResponse } from '@/bridge/client'

const router = useRouter()

const profile = ref<ProfileResponse | null>(null)
const loading = ref(true)
const error = ref('')

const formatDate = (dateString?: string | null) => {
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
    const response = await profileHandler({
      headers: {
        Authorization: `Bearer ${token}`
      }
    })

    if (response.data) {
      profile.value = response.data
    }
  } catch (err: any) {
    if (err.error?.message) {
      if (err.status === 401) {
        localStorage.removeItem('token')
        router.push('/login')
      } else {
        error.value = err.error.message
      }
    } else {
      error.value = 'Network error. Please make sure the backend server is running.'
    }
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchProfile()
})
</script>