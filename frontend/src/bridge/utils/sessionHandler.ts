/**
 * Session invalidation handler for API requests
 * Automatically handles session invalidation errors and redirects to login
 */

export interface ApiError {
  message: string
  status?: number
}

/**
 * Checks if an error is related to session invalidation
 */
export function isSessionInvalidationError(error: any): boolean {
  if (!error) return false
  
  // Check for 401 status code
  if (error.status === 401 || error.statusCode === 401) {
    return true
  }
  
  // Check for specific session-related error messages
  const message = error.message?.toLowerCase() || ''
  return (
    message.includes('session') ||
    message.includes('token expired') ||
    message.includes('invalid token') ||
    message.includes('session expired') ||
    message.includes('session has been invalidated') ||
    message.includes('session not found')
  )
}

/**
 * Handles session invalidation by clearing tokens and redirecting to login
 */
export function handleSessionInvalidation(isAdmin = false) {
  // Clear tokens
  if (isAdmin) {
    localStorage.removeItem('adminToken')
  } else {
    localStorage.removeItem('authToken')
  }
  
  // Show notification to user
  console.warn('Session has been invalidated. Please log in again.')
  
  // Redirect to appropriate login page
  if (isAdmin) {
    window.location.href = '/admin/login'
  } else {
    window.location.href = '/login'
  }
}

/**
 * Wraps an API call with session invalidation error handling
 */
export async function withSessionHandling<T>(
  apiCall: () => Promise<T>,
  isAdmin = false
): Promise<T> {
  try {
    return await apiCall()
  } catch (error: any) {
    if (isSessionInvalidationError(error)) {
      handleSessionInvalidation(isAdmin)
      throw new Error('Session invalidated. Please log in again.')
    }
    throw error
  }
}

/**
 * Creates an API error handler for fetch responses
 */
export async function handleApiResponse(response: Response, isAdmin = false): Promise<any> {
  if (!response.ok) {
    if (response.status === 401) {
      handleSessionInvalidation(isAdmin)
      throw new Error('Session invalidated. Please log in again.')
    }
    
    // Try to parse error response
    try {
      const errorData = await response.json()
      if (isSessionInvalidationError(errorData)) {
        handleSessionInvalidation(isAdmin)
        throw new Error('Session invalidated. Please log in again.')
      }
      throw new Error(errorData.message || `HTTP Error: ${response.status}`)
    } catch (parseError) {
      throw new Error(`HTTP Error: ${response.status} ${response.statusText}`)
    }
  }
  
  return response.json()
}

/**
 * Enhanced fetch function with session handling
 */
export async function apiRequest(
  url: string,
  options: RequestInit = {},
  isAdmin = false
): Promise<any> {
  const token = isAdmin 
    ? localStorage.getItem('adminToken')
    : localStorage.getItem('authToken')
  
  // Add authorization header if token exists
  const headers = {
    'Content-Type': 'application/json',
    ...(token && { 'Authorization': `Bearer ${token}` }),
    ...options.headers,
  }
  
  const response = await fetch(url, {
    ...options,
    headers,
  })
  
  return handleApiResponse(response, isAdmin)
}