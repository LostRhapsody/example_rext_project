<template>
  <div>
    <h2>Verify Email</h2>

    <div v-if="loading">
      <p>Email is being verified</p>
    </div>
    <div v-else>
      <div v-if="error">
        {{ error }}
      </div>
      <div v-else>
        {{ message }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
const verified = ref(false)
const loading = ref(false)
const error = ref("")
const message = ref("")
const GENERIC_ERROR = "Email verification failed. Please try again."
const route = useRoute()
const userId = route.query['id']?.toString() || ''
const handleVerification = async() => {
  loading.value = true
  console.log("running handle verification")
  try {
    console.log("Sending http request")
    const verificationData: VerifyEmailRequest = {
      user_id: userId,
    };
    const response = await verifyEmailHandler({
      body: verificationData,
    })
    if (response.error) {
      if(response.error.message){
        error.value = response.error.message
      } else {
        error.value = GENERIC_ERROR
      }
      return
    }

    if(response.data){
      message.value = response.data.message
      verified.value = response.data.success
    } else {
      error.value = GENERIC_ERROR
    }
  } catch(err: any){
    if(err.error?.message) {
      error.value = err.error.message
      console.error(err.error.message)
    } else {
      error.value = "Error communicating with server."
      console.error("Error communicating with server.")
    }
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  handleVerification()
})
</script>