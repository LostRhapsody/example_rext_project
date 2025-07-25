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
        <select v-model="filters.is_admin" @change="applyFilters" class="admin-filter">
          <option value="">All Users</option>
          <option value="true">Admins Only</option>
          <option value="false">Regular Users Only</option>
        </select>
      </div>
      <div class="action-buttons">
        <button @click="showCreateModal = true" class="create-btn">
          <span>➕</span> Create User
        </button>
        <button @click="refreshUsers" class="refresh-btn">
          <span>🔄</span> Refresh
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
        :paginationPageSize="filters.limit"
        :rowSelection="'single'"
        :animateRows="true"
        :domLayout="'autoHeight'"
        class="ag-theme-alpine"
        @grid-ready="onGridReady"
        @row-clicked="onRowClicked"
      />
    </div>

    <!-- Pagination Info -->
    <div class="pagination-info">
      <div class="pagination-stats">
        <span>Showing {{ paginationInfo.start }} to {{ paginationInfo.end }} of {{ paginationInfo.total }} users</span>
      </div>
      <div class="pagination-controls">
        <button 
          @click="changePage(filters.page - 1)" 
          :disabled="filters.page <= 1"
          class="page-btn"
        >
          Previous
        </button>
        <span class="page-info">Page {{ filters.page }} of {{ paginationInfo.totalPages }}</span>
        <button 
          @click="changePage(filters.page + 1)" 
          :disabled="filters.page >= paginationInfo.totalPages"
          class="page-btn"
        >
          Next
        </button>
      </div>
    </div>

    <!-- Create User Modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click="closeCreateModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Create New User</h3>
          <button @click="closeCreateModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
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
              <label class="checkbox-label">
                <input
                  v-model="createForm.is_admin"
                  type="checkbox"
                />
                Admin privileges
              </label>
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
              <label class="checkbox-label">
                <input
                  v-model="editForm.is_admin"
                  type="checkbox"
                />
                Admin privileges
              </label>
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
              <span>{{ formatTimestamp(selectedUser.created_at) }}</span>
            </div>
            <div class="detail-item">
              <label>Admin Status:</label>
              <span :class="selectedUser.is_admin ? 'admin-badge' : 'user-badge'">
                {{ selectedUser.is_admin ? 'Admin' : 'Regular User' }}
              </span>
            </div>
          </div>
          <div class="detail-actions">
            <button @click="openEditModal" class="edit-btn">Edit User</button>
            <button @click="openDeleteModal" class="delete-btn">Delete User</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { AgGridVue } from 'ag-grid-vue3'
import 'ag-grid-community/styles/ag-grid.css'
import 'ag-grid-community/styles/ag-theme-alpine.css'
import { 
  getUsersHandler, 
  createUserHandler, 
  updateUserHandler, 
  deleteUserHandler,
  getUserHandler 
} from '@/bridge/client'

interface User {
  id: string
  email: string
  is_admin?: boolean | null
  created_at?: string | null
}

interface CreateUserForm {
  email: string
  password: string
  is_admin: boolean
}

interface UpdateUserForm {
  email: string
  password: string
  is_admin: boolean
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
  limit: 20,
  search: '',
  is_admin: null as boolean | null
})

const showCreateModal = ref(false)
const showEditModal = ref(false)
const showDeleteModal = ref(false)
const selectedUser = ref<User | null>(null)

const createForm = reactive<CreateUserForm>({
  email: '',
  password: '',
  is_admin: false
})

const editForm = reactive<UpdateUserForm>({
  email: '',
  password: '',
  is_admin: false
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
    headerName: 'Admin',
    field: 'is_admin',
    width: 100,
    cellRenderer: (params: any) => {
      return params.value ? '✅' : '❌'
    },
    sortable: true,
    filter: true
  },
  {
    headerName: 'Created',
    field: 'created_at',
    width: 150,
    valueFormatter: (params: any) => formatTimestamp(params.value),
    sortable: true,
    filter: true
  },
  {
    headerName: 'Actions',
    width: 200,
    cellRenderer: (params: any) => {
      return `
        <div style="display: flex; gap: 8px;">
          <button onclick="window.viewUser('${params.data.id}')" style="background: #007bff; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">View</button>
          <button onclick="window.editUser('${params.data.id}')" style="background: #ffc107; color: black; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">Edit</button>
          <button onclick="window.deleteUser('${params.data.id}')" style="background: #dc3545; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">Delete</button>
        </div>
      `
    }
  }
])

const defaultColDef = {
  resizable: true,
  sortable: true,
  filter: true,
  floatingFilter: true
}

const creating = ref(false)
const updating = ref(false)
const deleting = ref(false)

// Modal states
const showDetailModal = ref(false)

const fetchUsers = async () => {
  loading.value = true
  
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const query: any = {
      page: filters.page,
      limit: filters.limit
    }

    if (filters.search) query.search = filters.search
    if (filters.is_admin !== null) query.is_admin = filters.is_admin

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
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await createUserHandler({
      body: {
        email: createForm.email,
        password: createForm.password,
        is_admin: createForm.is_admin
      },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showCreateModal.value = false
      resetCreateForm()
      fetchUsers()
    }
  } catch (error) {
    console.error('Error creating user:', error)
  } finally {
    creating.value = false
  }
}

const updateUser = async () => {
  updating.value = true
  if (!selectedUser.value) return

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const updateData: any = {
      email: editForm.email,
      is_admin: editForm.is_admin
    }

    if (editForm.password) {
      updateData.password = editForm.password
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
    }
  } catch (error) {
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

    const response = await deleteUserHandler({
      path: { id: selectedUser.value.id },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showDeleteModal.value = false
      selectedUser.value = null
      fetchUsers()
    }
  } catch (error) {
    console.error('Error deleting user:', error)
  } finally {
    deleting.value = false
  }
}

const refreshUsers = () => {
  fetchUsers()
}

const previousPage = () => {
  if (filters.page > 1) {
    filters.page--
    fetchUsers()
  }
}

const nextPage = () => {
  if (filters.page < paginationInfo.value.totalPages) {
    filters.page++
    fetchUsers()
  }
}

const onGridReady = (params: any) => {
  // Expose functions globally for AG Grid button clicks
  ;(window as any).viewUser = viewUser
  ;(window as any).editUser = editUser
  ;(window as any).deleteUser = deleteUser
}

const onRowClicked = (params: any) => {
  selectedUser.value = params.data
  showDetailModal.value = true
}

// Modal functions
const closeCreateModal = () => {
  showCreateModal.value = false
  resetCreateForm()
}

const closeEditModal = () => {
  showEditModal.value = false
  selectedUser.value = null
}

const closeDeleteModal = () => {
  showDeleteModal.value = false
  selectedUser.value = null
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

const editUser = async (userId: string) => {
  await viewUser(userId)
}

const deleteUser = (userId: string) => {
  selectedUser.value = users.value.find(u => u.id === userId) || null
  showDeleteModal.value = true
}

const populateEditForm = (user: User) => {
  editForm.email = user.email
  editForm.password = ''
  editForm.is_admin = user.is_admin || false
}

const resetCreateForm = () => {
  createForm.email = ''
  createForm.password = ''
  createForm.is_admin = false
}

const resetEditForm = () => {
  editForm.email = ''
  editForm.password = ''
  editForm.is_admin = false
  selectedUser.value = null
}

const applyFilters = () => {
  filters.page = 1
  fetchUsers()
}

const clearFilters = () => {
  filters.search = ''
  filters.is_admin = null
  filters.page = 1
  fetchUsers()
}

const changePage = (newPage: number) => {
  if (newPage >= 1 && newPage <= paginationInfo.value.totalPages) {
    filters.page = newPage
    fetchUsers()
  }
}

onMounted(() => {
  fetchUsers()
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
.refresh-btn {
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

.form-group input {
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
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
</style> 