<template>
  <div class="admin-users">
    <div class="users-header">
      <h1>User Management</h1>
      <p>Manage user accounts and permissions</p>
    </div>

    <!-- Actions Bar -->
    <div class="actions-bar">
      <div class="search-section">
        <input
          v-model="filters.search"
          type="text"
          placeholder="Search users by email..."
          @input="applyFilters"
          class="search-input"
        />
        <select v-model="filters.role_id" @change="applyFilters" class="role-filter">
          <option value="">All Users</option>
          <option value="1">Admins Only</option>
          <option value="2">Regular Users Only</option>
        </select>
      </div>
      <div class="action-buttons">
        <button @click="showCreateModal = true" class="create-btn">
          <span>➕</span> Create User
        </button>
        <button @click="refreshUsers" class="refresh-btn">
          <span>🔄</span> Refresh
        </button>
        <button @click="exportToCsv" class="export-btn">
          <span>📊</span> Export CSV
        </button>
      </div>
    </div>

    <!-- AG Grid -->
    <div class="grid-container">
      <ag-grid-vue
        :columnDefs="columnDefs"
        :rowData="users"
        :defaultColDef="defaultColDef"
        :pagination="true"
        :paginationPageSize="20"
        :rowSelection="rowSelection"
        :animateRows="true"
        :domLayout="'autoHeight'"
        @row-clicked="onRowClicked"
        @grid-ready="onGridReady"
      />
    </div>

    <!-- Create User Modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click="closeCreateModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Create New User</h3>
          <button @click="closeCreateModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div v-if="createError" class="error-message">
            {{ createError }}
          </div>
          <form @submit.prevent="createUser" class="user-form">
            <div class="form-group">
              <label for="create-email">Email *</label>
              <input
                id="create-email"
                v-model="createForm.email"
                type="email"
                required
                placeholder="user@example.com"
              />
            </div>
            <div class="form-group">
              <label for="create-password">Password *</label>
              <input
                id="create-password"
                v-model="createForm.password"
                type="password"
                required
                placeholder="Enter password"
                minlength="6"
              />
            </div>
            <div class="form-group">
              <label for="create-role">Role</label>
              <select
                id="create-role"
                v-model="createForm.role_id"
                class="role-select"
              >
                <option :value="null">No Role</option>
                <option
                  v-for="role in roles"
                  :key="role.id"
                  :value="role.id"
                >
                  {{ role.name }} - {{ role.description }}
                </option>
              </select>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeCreateModal" class="cancel-btn">Cancel</button>
              <button type="submit" :disabled="creating" class="submit-btn">
                {{ creating ? 'Creating...' : 'Create User' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Edit User Modal -->
    <div v-if="showEditModal && selectedUser" class="modal-overlay" @click="closeEditModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Edit User</h3>
          <button @click="closeEditModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div v-if="updateError" class="error-message">
            {{ updateError }}
          </div>
          <form @submit.prevent="updateUser" class="user-form">
            <div class="form-group">
              <label for="edit-email">Email *</label>
              <input
                id="edit-email"
                v-model="editForm.email"
                type="email"
                required
                placeholder="user@example.com"
              />
            </div>
            <div class="form-group">
              <label for="edit-password">Password</label>
              <input
                id="edit-password"
                v-model="editForm.password"
                type="password"
                placeholder="Leave blank to keep current password"
                minlength="6"
              />
              <small>Leave blank to keep current password</small>
            </div>
            <div class="form-group">
              <label for="edit-role">Role</label>
              <select
                id="edit-role"
                v-model="editForm.role_id"
                class="role-select"
              >
                <option :value="null">No Role</option>
                <option
                  v-for="role in roles"
                  :key="role.id"
                  :value="role.id"
                >
                  {{ role.name }} - {{ role.description }}
                </option>
              </select>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeEditModal" class="cancel-btn">Cancel</button>
              <button type="submit" :disabled="updating" class="submit-btn">
                {{ updating ? 'Updating...' : 'Update User' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteModal && selectedUser" class="modal-overlay" @click="closeDeleteModal">
      <div class="modal-content delete-modal" @click.stop>
        <div class="modal-header">
          <h3>Delete User</h3>
          <button @click="closeDeleteModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div v-if="deleteError" class="error-message">
            {{ deleteError }}
          </div>
          <div class="delete-warning">
            <p>Are you sure you want to delete this user?</p>
            <div class="user-info">
              <strong>Email:</strong> {{ selectedUser.email }}
              <br>
              <strong>ID:</strong> {{ selectedUser.id }}
            </div>
            <p class="warning-text">This action cannot be undone.</p>
          </div>
          <div class="form-actions">
            <button @click="closeDeleteModal" class="cancel-btn">Cancel</button>
            <button @click="confirmDelete" :disabled="deleting" class="delete-btn">
              {{ deleting ? 'Deleting...' : 'Delete User' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- User Detail Modal -->
    <div v-if="showDetailModal && selectedUser" class="modal-overlay" @click="closeDetailModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>User Details</h3>
          <button @click="closeDetailModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="user-detail-grid">
            <div class="detail-item">
              <label>ID:</label>
              <span>{{ selectedUser.id }}</span>
            </div>
            <div class="detail-item">
              <label>Email:</label>
              <span>{{ selectedUser.email }}</span>
            </div>
            <div class="detail-item">
              <label>Created At:</label>
              <span>{{ formatTimestamp(selectedUser.created_at || '') }}</span>
            </div>
            <div class="detail-item">
              <label>Role:</label>
              <span>{{ selectedUser.role_name || 'No Role' }}</span>
            </div>
          </div>
          <div class="detail-actions">
            <button @click="openEditModal" class="edit-btn">Edit User</button>
            <button @click="openSessionModal(selectedUser)" class="session-btn">Manage Sessions</button>
            <button @click="openDeleteModal" class="delete-btn">Delete User</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Session Management Modal -->
    <div v-if="showSessionModal && selectedUser" class="modal-overlay" @click="closeSessionModal">
      <div class="modal-content session-modal" @click.stop>
        <div class="modal-header">
          <h3>Manage Sessions for {{ selectedUser.email }}</h3>
          <button @click="closeSessionModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="session-actions">
            <button 
              @click="invalidateAllUserSessions(selectedUser.id)" 
              class="logout-all-btn"
            >
              🚪 Log Out All Sessions
            </button>
            <button 
              @click="fetchUserSessions(selectedUser.id)"
              class="refresh-btn"
              :disabled="sessionsLoading"
            >
              🔄 Refresh
            </button>
          </div>

          <div v-if="sessionsLoading" class="sessions-loading">
            Loading sessions...
          </div>

          <div v-else-if="userSessions.length === 0" class="no-sessions">
            No active sessions found
          </div>

          <div v-else class="sessions-list">
            <div 
              v-for="session in userSessions" 
              :key="session.id"
              class="session-item"
            >
              <div class="session-info">
                <div class="session-device">
                  <span class="device-icon">
                    {{ parseUserAgent(session.device_info).includes('Mobile') ? '📱' : '💻' }}
                  </span>
                  <span class="device-name">{{ parseUserAgent(session.device_info) }}</span>
                </div>
                <div class="session-details">
                  <div class="session-ip" v-if="session.ip_address">
                    📍 {{ session.ip_address }}
                  </div>
                  <div class="session-time">
                    🕐 Last active: {{ formatTimeSince(session.last_activity) }}
                  </div>
                  <div class="session-created">
                    📅 Created: {{ formatTimestamp(session.created_at) }}
                  </div>
                </div>
              </div>
              <div class="session-actions-item">
                <button 
                  @click="invalidateSession(session.id)"
                  class="logout-session-btn"
                >
                  🚪 Log Out
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AgGridVue } from 'ag-grid-vue3'
import { apiRequest, withSessionHandling } from '@/bridge/utils/sessionHandler'

interface User {
  id: string
  email: string
  created_at?: string | null
  role_id?: number | null
  role_name?: string | null
}

interface Role {
  id: number
  name: string
  description?: string | null
  permissions: string[]
}

interface CreateUserForm {
  email: string
  password: string
  role_id?: number | null
}

interface UpdateUserForm {
  email: string
  password: string
  role_id?: number | null
}

interface UserSession {
  id: string
  user_id: string
  device_info: string
  ip_address?: string | null
  created_at: string
  last_activity: string
  expires_at: string
  is_current: boolean
}

interface PaginationInfo {
  page: number
  limit: number
  total: number
  totalPages: number
  start: number
  end: number
}

const loading = ref(false)
const users = ref<User[]>([])
const roles = ref<Role[]>([])
const gridApi = ref<any>(null)
const paginationInfo = ref<PaginationInfo>({
  page: 1,
  limit: 20,
  total: 0,
  totalPages: 0,
  start: 0,
  end: 0
})

const filters = reactive({
  page: 1,
  limit: 9999,
  search: '',
  role_id: ''
})

const showCreateModal = ref(false)
const showSessionModal = ref(false)
const userSessions = ref<UserSession[]>([])
const sessionsLoading = ref(false)
const showEditModal = ref(false)
const showDeleteModal = ref(false)
const selectedUser = ref<User | null>(null)

const createForm = reactive<CreateUserForm>({
  email: '',
  password: '',
  role_id: null
})

const editForm = reactive<UpdateUserForm>({
  email: '',
  password: '',
  role_id: null
})

const columnDefs = ref([
  {
    headerName: 'ID',
    field: 'id',
    width: 120,
    sortable: true,
    filter: true
  },
  {
    headerName: 'Email',
    field: 'email',
    flex: 1,
    sortable: true,
    filter: true
  },
  {
    headerName: 'Roles',
    field: 'role_name',
    flex: 1,
    cellRenderer: (params: any) => {
      return params.value ? params.value : 'No Role'
    },
    sortable: true,
    filter: true
  },
  {
    headerName: 'Created',
    field: 'created_at',
    flex: 1,
    valueFormatter: (params: any) => formatTimestamp(params.value),
    sortable: true,
    filter: true
  }
])

const defaultColDef = {
  resizable: true,
  sortable: true,
  filter: true,
  floatingFilter: true
}

const rowSelection = {
  mode: 'singleRow',
} as any; // AG Grid type definitions are overly strict

const creating = ref(false)
const updating = ref(false)
const deleting = ref(false)
const deleteError = ref('')
const createError = ref('')
const updateError = ref('')

// Modal states
const showDetailModal = ref(false)

const fetchUsers = async () => {
  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const query: GetUsersHandlerData["query"] = {
      page: filters.page,
      limit: filters.limit
    }

    if (filters.search) query.search = filters.search

    const response = await getUsersHandler({
      query,
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      users.value = response.data.data || []
      const pagination = response.data.pagination
      paginationInfo.value = {
        page: pagination.page,
        limit: pagination.limit,
        total: pagination.total,
        totalPages: pagination.total_pages,
        start: (pagination.page - 1) * pagination.limit + 1,
        end: Math.min(pagination.page * pagination.limit, pagination.total)
      }
    }
  } catch (error) {
    console.error('Error fetching users:', error)
  } finally {
    loading.value = false
  }
}

const createUser = async () => {
  creating.value = true
  createError.value = ''
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await createUserHandler({
      body: {
        email: createForm.email,
        password: createForm.password,
        role_id: createForm.role_id
      },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showCreateModal.value = false
      resetCreateForm()
      fetchUsers()
    } else if (response.error) {
      createError.value = response.error.message || 'Failed to create user'
    }
  } catch (error: any) {
    createError.value = error.message || 'Failed to create user'
    console.error('Error creating user:', error)
  } finally {
    creating.value = false
  }
}

const updateUser = async () => {
  updating.value = true
  updateError.value = ''
  if (!selectedUser.value) return

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const updateData: any = {
      email: editForm.email,
      role_id: editForm.role_id
    }

    if (editForm.password) {
      updateData.password = editForm.password
    }

    if (editForm.role_id) {
      updateData.role_id = editForm.role_id
    }

    const response = await updateUserHandler({
      path: { id: selectedUser.value.id },
      body: updateData,
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showEditModal.value = false
      resetEditForm()
      fetchUsers()
    } else if (response.error) {
      updateError.value = response.error.message || 'Failed to update user'
    }
  } catch (error: any) {
    updateError.value = error.message || 'Failed to update user'
    console.error('Error updating user:', error)
  } finally {
    updating.value = false
  }
}

const confirmDelete = async () => {
  deleting.value = true
  if (!selectedUser.value) return

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await deleteUserHandler<false>({
      path: { id: selectedUser.value.id },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showDeleteModal.value = false
      selectedUser.value = null
      deleteError.value = ''
      fetchUsers()
    } else if (response.error) {
      console.error('Error deleting user:', response.error)
      deleteError.value = response.error.message || 'Failed to delete user'
    }
  } catch (error: any) {
    console.error('Error deleting user:', error)
    deleteError.value = error.message || 'Failed to delete user'
  } finally {
    deleting.value = false
  }
}

const refreshUsers = () => {
  fetchUsers()
}

const onRowClicked = (params: any) => {
  selectedUser.value = params.data
  showDetailModal.value = true
}

// Modal functions
const closeCreateModal = () => {
  showCreateModal.value = false
  resetCreateForm()
  createError.value = ''
}

const closeEditModal = () => {
  showEditModal.value = false
  selectedUser.value = null
  updateError.value = ''
}

const closeDeleteModal = () => {
  showDeleteModal.value = false
  selectedUser.value = null
  deleteError.value = ''
}

const closeDetailModal = () => {
  showDetailModal.value = false
  selectedUser.value = null
}

const openEditModal = () => {
  if (selectedUser.value) {
    populateEditForm(selectedUser.value)
    showDetailModal.value = false
    showEditModal.value = true
  }
}

const openDeleteModal = () => {
  showDetailModal.value = false
  showDeleteModal.value = true
}

const formatTimestamp = (timestamp?: string) => {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return timestamp
  }
}

const viewUser = async (userId: string) => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await getUserHandler({
      path: { id: userId },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      selectedUser.value = response.data
      showEditModal.value = true
      populateEditForm(response.data)
    }
  } catch (error) {
    console.error('Error fetching user:', error)
  }
}

const populateEditForm = (user: User) => {
  editForm.email = user.email
  editForm.password = ''
  editForm.role_id = user.role_id ?? null
}

const resetCreateForm = () => {
  createForm.email = ''
  createForm.password = ''
  createForm.role_id = null
}

const resetEditForm = () => {
  editForm.email = ''
  editForm.password = ''
  editForm.role_id = null
  selectedUser.value = null
}

const applyFilters = () => {
  filters.page = 1
  fetchUsers()
}

const onGridReady = (params: any) => {
  gridApi.value = params.api
}

const exportToCsv = () => {
  if (!gridApi.value) return

  const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-')
  const fileName = `users-${timestamp}.csv`

  gridApi.value.exportDataAsCsv({
    fileName,
    processCellCallback: (params: any) => {
      // Handle special characters that could cause CSV injection
      const value = params.value?.toString() || ''
      if (value.startsWith('+') || value.startsWith('-') || value.startsWith('=') || value.startsWith('@')) {
        return `"${value}"`
      }
      return value
    }
  })
}

const fetchRoles = async () => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    // For now, we'll use a simple fetch until the API client is generated
    const response = await getRolesHandler({
      query: {
        page: 1,
        limit: 9999
      },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      roles.value = response.data.data || []
    } else if (response.error) {
      console.error('Error fetching roles:', response.error)
    }
  } catch (error) {
    console.error('Error fetching roles:', error)
  }
}

// Session Management Functions
const fetchUserSessions = async (userId: string) => {
  if (!userId) return
  
  sessionsLoading.value = true
  try {
    const sessions = await withSessionHandling(
      () => apiRequest(`http://localhost:3000/api/v1/admin/users/${userId}/sessions`, {}, true),
      true
    )
    userSessions.value = sessions || []
  } catch (error) {
    console.error('Error fetching user sessions:', error)
    userSessions.value = []
  } finally {
    sessionsLoading.value = false
  }
}

const invalidateSession = async (sessionId: string) => {
  try {
    await withSessionHandling(
      () => apiRequest(`http://localhost:3000/api/v1/admin/sessions/${sessionId}`, { method: 'DELETE' }, true),
      true
    )
    
    // Refresh sessions list
    if (selectedUser.value) {
      await fetchUserSessions(selectedUser.value.id)
    }
  } catch (error) {
    console.error('Error invalidating session:', error)
  }
}

const invalidateAllUserSessions = async (userId: string) => {
  try {
    await withSessionHandling(
      () => apiRequest(`http://localhost:3000/api/v1/admin/users/${userId}/sessions`, { method: 'DELETE' }, true),
      true
    )
    
    // Refresh sessions list
    await fetchUserSessions(userId)
  } catch (error) {
    console.error('Error invalidating all user sessions:', error)
  }
}

const openSessionModal = async (user: User) => {
  selectedUser.value = user
  showSessionModal.value = true
  await fetchUserSessions(user.id)
}

const closeSessionModal = () => {
  showSessionModal.value = false
  userSessions.value = []
  selectedUser.value = null
}

const parseUserAgent = (userAgent: string): string => {
  if (!userAgent) return 'Unknown Device'
  
  // Simple user agent parsing
  if (userAgent.includes('Mobile') || userAgent.includes('Android') || userAgent.includes('iPhone')) {
    if (userAgent.includes('Chrome')) return 'Mobile Chrome'
    if (userAgent.includes('Safari')) return 'Mobile Safari'
    if (userAgent.includes('Firefox')) return 'Mobile Firefox'
    return 'Mobile Browser'
  }
  
  if (userAgent.includes('Chrome')) return 'Chrome Browser'
  if (userAgent.includes('Firefox')) return 'Firefox Browser'
  if (userAgent.includes('Safari')) return 'Safari Browser'
  if (userAgent.includes('Edge')) return 'Edge Browser'
  
  return 'Desktop Browser'
}

const formatTimeSince = (timestamp: string): string => {
  const now = new Date()
  const time = new Date(timestamp)
  const diffMs = now.getTime() - time.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins} min ago`
  if (diffHours < 24) return `${diffHours} hour${diffHours !== 1 ? 's' : ''} ago`
  return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`
}

onMounted(() => {
  fetchUsers()
  fetchRoles()
})
</script>

<style scoped>
.admin-users {
  padding: 2rem;
}

.users-header {
  margin-bottom: 2rem;
}

.users-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.users-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.actions-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  margin-bottom: 2rem;
}

.search-section {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.search-input {
  padding: 0.5rem 1rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
  min-width: 250px;
}

.admin-filter {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
}

.action-buttons {
  display: flex;
  gap: 1rem;
}

.create-btn,
.refresh-btn,
.export-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.create-btn {
  background: #28a745;
  color: white;
}

.create-btn:hover {
  background: #218838;
}

.refresh-btn {
  background: #667eea;
  color: white;
}

.refresh-btn:hover {
  background: #5a6fd8;
}

.export-btn {
  background: #28a745;
  color: white;
}

.export-btn:hover {
  background: #218838;
}

.grid-container {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
  margin-bottom: 1rem;
}

.ag-theme-alpine {
  height: 600px;
  width: 100%;
}

.pagination-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: white;
  padding: 1rem 1.5rem;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.pagination-stats {
  color: #666;
  font-size: 0.9rem;
}

.pagination-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.page-btn {
  padding: 0.5rem 1rem;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.page-btn:hover:not(:disabled) {
  background: #f8f9fa;
}

.page-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.page-info {
  font-size: 0.9rem;
  color: #666;
}

/* Modal Styles */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  max-width: 500px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-content.delete-modal {
  max-width: 400px;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.modal-header h3 {
  margin: 0;
  color: #333;
}

.close-btn {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #666;
}

.modal-body {
  padding: 1.5rem;
}

.user-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
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

.form-group input,
.form-group select {
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
}

.role-select {
  width: 100%;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  width: auto;
  margin: 0;
}

.form-group small {
  color: #666;
  font-size: 0.8rem;
}

.form-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
  margin-top: 1rem;
}

.cancel-btn,
.submit-btn {
  padding: 0.75rem 1.5rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.cancel-btn {
  background: #f8f9fa;
  color: #666;
  border: 1px solid #ddd;
}

.cancel-btn:hover {
  background: #e9ecef;
}

.submit-btn {
  background: #667eea;
  color: white;
}

.submit-btn:hover:not(:disabled) {
  background: #5a6fd8;
}

.submit-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.delete-btn {
  background: #dc3545;
  color: white;
}

.delete-btn:hover:not(:disabled) {
  background: #c82333;
}

.delete-warning {
  text-align: center;
  margin-bottom: 1.5rem;
}

.user-info {
  background: #f8f9fa;
  padding: 1rem;
  border-radius: 6px;
  margin: 1rem 0;
  text-align: left;
}

.warning-text {
  color: #dc3545;
  font-weight: 600;
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 0.75rem;
  border-radius: 6px;
  margin-bottom: 1rem;
  border: 1px solid #f5c6cb;
  font-size: 0.9rem;
}

.user-detail-grid {
  display: grid;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: #f8f9fa;
  border-radius: 6px;
}

.detail-item label {
  font-weight: 600;
  color: #333;
}

.detail-item span {
  color: #666;
}

.admin-badge {
  background: #28a745;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

.user-badge {
  background: #6c757d;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

.detail-actions {
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.edit-btn {
  background: #ffc107;
  color: #212529;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.edit-btn:hover {
  background: #e0a800;
}

/* AG Grid custom styles */
:deep(.action-buttons) {
  display: flex;
  gap: 0.5rem;
}

:deep(.view-btn),
:deep(.edit-btn),
:deep(.delete-btn) {
  padding: 0.25rem 0.5rem;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.8rem;
  transition: all 0.2s;
}

:deep(.view-btn) {
  background: #17a2b8;
  color: white;
}

:deep(.view-btn:hover) {
  background: #138496;
}

:deep(.edit-btn) {
  background: #ffc107;
  color: #212529;
}

:deep(.edit-btn:hover) {
  background: #e0a800;
}

:deep(.delete-btn) {
  background: #dc3545;
  color: white;
}

:deep(.delete-btn:hover) {
  background: #c82333;
}

@media (max-width: 768px) {
  .admin-users {
    padding: 1rem;
  }

  .actions-bar {
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;
  }

  .search-section {
    flex-direction: column;
    align-items: stretch;
  }

  .search-input {
    min-width: auto;
  }

  .pagination-info {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .form-actions {
    flex-direction: column;
  }

  .detail-actions {
    flex-direction: column;
  }
}

/* Session Management Styles */
.session-modal {
  max-width: 700px;
}

.session-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.session-btn:hover {
  background: #0056b3;
}

.session-actions {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
  padding-bottom: 1rem;
  border-bottom: 1px solid #e0e0e0;
}

.logout-all-btn {
  background: #dc3545;
  color: white;
  border: none;
  padding: 0.75rem 1.5rem;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.logout-all-btn:hover {
  background: #c82333;
}

.refresh-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.sessions-loading,
.no-sessions {
  text-align: center;
  padding: 2rem;
  color: #666;
  font-size: 1.1rem;
}

.sessions-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.session-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: #f8f9fa;
  transition: background 0.2s;
}

.session-item:hover {
  background: #e9ecef;
}

.session-info {
  flex: 1;
}

.session-device {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.device-icon {
  font-size: 1.2rem;
}

.device-name {
  color: #333;
}

.session-details {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  font-size: 0.9rem;
  color: #666;
}

.session-ip,
.session-time,
.session-created {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.session-actions-item {
  margin-left: 1rem;
}

.logout-session-btn {
  background: #dc3545;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.2s;
}

.logout-session-btn:hover {
  background: #c82333;
}

@media (max-width: 768px) {
  .session-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 1rem;
  }

  .session-actions-item {
    margin-left: 0;
  }

  .session-actions {
    flex-direction: column;
  }
}
</style>