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

    // Check if there's an error in the response
    if (response.error) {
      if (response.error.message) {
        if (response.response?.status === 401) {
          localStorage.removeItem('token')
          router.push('/login')
        } else {
          error.value = response.error.message
        }
      } else {
        error.value = 'Failed to load profile. Please try again.'
      }
      return
    }

    // Check if we have successful data
    if (response.data) {
      profile.value = response.data
    } else {
      error.value = 'Failed to load profile. Please try again.'
    }
  } catch (err: any) {
    // This catch block will only trigger for network errors or other exceptions
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