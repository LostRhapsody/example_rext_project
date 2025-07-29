<template>
  <div class="admin-roles">
    <div class="roles-header">
      <h1>Role Management</h1>
      <p>Manage user roles and permissions</p>
    </div>

    <!-- Actions Bar -->
    <div class="actions-bar">
      <div class="search-section">
        <input
          v-model="filters.search"
          type="text"
          placeholder="Search roles by name..."
          @input="applyFilters"
          class="search-input"
        />
      </div>
      <div class="action-buttons">
        <button @click="showCreateModal = true" class="create-btn">
          <span>➕</span> Create Role
        </button>
        <button @click="refreshRoles" class="refresh-btn">
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
        :rowData="roles"
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

    <!-- Create Role Modal -->
    <div v-if="showCreateModal" class="modal-overlay" @click="closeCreateModal">
      <div class="modal-content create-modal" @click.stop>
        <div class="modal-header">
          <h3>Create New Role</h3>
          <button @click="closeCreateModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <form @submit.prevent="createRole" class="role-form">
            <div class="form-group">
              <label for="create-name">Name *</label>
              <input
                id="create-name"
                v-model="createForm.name"
                type="text"
                required
                placeholder="Role name"
              />
            </div>
            <div class="form-group">
              <label for="create-description">Description</label>
              <textarea
                id="create-description"
                v-model="createForm.description"
                placeholder="Role description"
                rows="3"
              />
            </div>
            <div class="form-group">
              <label>Permissions</label>
              <div class="permissions-grid">
                <div v-for="permission in availablePermissions" :key="permission.value" class="permission-item">
                  <label class="checkbox-label">
                    <input
                      type="checkbox"
                      :value="permission.value"
                      v-model="createForm.permissions"
                    />
                    <span class="permission-name">{{ permission.label }}</span>
                    <span class="permission-description">{{ permission.description }}</span>
                  </label>
                </div>
              </div>
              <div class="custom-permissions">
                <label for="create-custom-permission">Custom Permissions (comma-separated)</label>
                <input
                  id="create-custom-permission"
                  v-model="customPermissionInput"
                  type="text"
                  placeholder="custom:permission1, custom:permission2"
                  @input="updateCustomPermissions"
                />
              </div>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeCreateModal" class="cancel-btn">Cancel</button>
              <button type="submit" :disabled="creating" class="submit-btn">
                {{ creating ? 'Creating...' : 'Create Role' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Edit Role Modal -->
    <div v-if="showEditModal && selectedRole" class="modal-overlay" @click="closeEditModal">
      <div class="modal-content edit-modal" @click.stop>
        <div class="modal-header">
          <h3>Edit Role</h3>
          <button @click="closeEditModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <form @submit.prevent="updateRole" class="role-form">
            <div class="form-group">
              <label for="edit-name">Name *</label>
              <input
                id="edit-name"
                v-model="editForm.name"
                type="text"
                required
                placeholder="Role name"
              />
            </div>
            <div class="form-group">
              <label for="edit-description">Description</label>
              <textarea
                id="edit-description"
                v-model="editForm.description"
                placeholder="Role description"
                rows="3"
              />
            </div>
            <div class="form-group">
              <label>Permissions</label>
              <div class="permissions-grid">
                <div v-for="permission in availablePermissions" :key="permission.value" class="permission-item">
                  <label class="checkbox-label">
                    <input
                      type="checkbox"
                      :value="permission.value"
                      v-model="editForm.permissions"
                    />
                    <span class="permission-name">{{ permission.label }}</span>
                    <span class="permission-description">{{ permission.description }}</span>
                  </label>
                </div>
              </div>
              <div class="custom-permissions">
                <label for="edit-custom-permission">Custom Permissions (comma-separated)</label>
                <input
                  id="edit-custom-permission"
                  v-model="editCustomPermissionInput"
                  type="text"
                  placeholder="custom:permission1, custom:permission2"
                  @input="updateEditCustomPermissions"
                />
              </div>
            </div>
            <div class="form-actions">
              <button type="button" @click="closeEditModal" class="cancel-btn">Cancel</button>
              <button type="submit" :disabled="updating" class="submit-btn">
                {{ updating ? 'Updating...' : 'Update Role' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteModal && selectedRole" class="modal-overlay" @click="closeDeleteModal">
      <div class="modal-content delete-modal" @click.stop>
        <div class="modal-header">
          <h3>Delete Role</h3>
          <button @click="closeDeleteModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div v-if="deleteError" class="error-message">
            {{ deleteError }}
          </div>
          <div class="delete-warning">
            <p>Are you sure you want to delete this role?</p>
            <div class="role-info">
              <strong>Name:</strong> {{ selectedRole.name }}
              <br>
              <strong>Description:</strong> {{ selectedRole.description || 'No description' }}
              <br>
              <strong>ID:</strong> {{ selectedRole.id }}
            </div>
            <p class="warning-text">This action cannot be undone.</p>
          </div>
          <div class="form-actions">
            <button @click="closeDeleteModal" class="cancel-btn">Cancel</button>
            <button @click="confirmDelete" :disabled="deleting" class="delete-btn">
              {{ deleting ? 'Deleting...' : 'Delete Role' }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Role Detail Modal -->
    <div v-if="showDetailModal && selectedRole" class="modal-overlay" @click="closeDetailModal">
      <div class="modal-content detail-modal" @click.stop>
        <div class="modal-header">
          <h3>Role Details</h3>
          <button @click="closeDetailModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="role-detail-grid">
            <div class="detail-item">
              <label>ID:</label>
              <span>{{ selectedRole.id }}</span>
            </div>
            <div class="detail-item">
              <label>Name:</label>
              <span>{{ selectedRole.name }}</span>
            </div>
            <div class="detail-item">
              <label>Description:</label>
              <span>{{ selectedRole.description || 'No description' }}</span>
            </div>
            <div class="detail-item">
              <label>Created At:</label>
              <span>{{ formatTimestamp(selectedRole.created_at || '') }}</span>
            </div>
            <div class="detail-item">
              <label>Updated At:</label>
              <span>{{ formatTimestamp(selectedRole.updated_at || '') }}</span>
            </div>
            <div class="detail-item permissions-detail">
              <label>Permissions:</label>
              <div class="permissions-list">
                <span 
                  v-for="permission in selectedRole.permissions" 
                  :key="permission" 
                  class="permission-tag"
                >
                  {{ permission }}
                </span>
              </div>
            </div>
          </div>
          <div class="detail-actions">
            <button @click="openEditModal" class="edit-btn">Edit Role</button>
            <button @click="openDeleteModal" class="delete-btn">Delete Role</button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AgGridVue } from 'ag-grid-vue3'
import {
  getRolesHandler,
  createRoleHandler,
  updateRoleHandler,
  deleteRoleHandler,
} from '@/bridge/client'
import type { GetRolesHandlerData } from '@/bridge/client/types.gen'

interface Role {
  id: number
  name: string
  description?: string | null
  permissions: string[]
  created_at?: string | null
  updated_at?: string | null
}

interface CreateRoleForm {
  name: string
  description: string
  permissions: string[]
}

interface UpdateRoleForm {
  name: string
  description: string
  permissions: string[]
}

interface PaginationInfo {
  page: number
  limit: number
  total: number
  totalPages: number
  start: number
  end: number
}

interface PermissionOption {
  value: string
  label: string
  description: string
  category: string
}

const loading = ref(false)
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
  search: ''
})

const showCreateModal = ref(false)
const showEditModal = ref(false)
const showDeleteModal = ref(false)
const showDetailModal = ref(false)
const selectedRole = ref<Role | null>(null)

const createForm = reactive<CreateRoleForm>({
  name: '',
  description: '',
  permissions: []
})

const editForm = reactive<UpdateRoleForm>({
  name: '',
  description: '',
  permissions: []
})

const customPermissionInput = ref('')
const editCustomPermissionInput = ref('')

// Define available permissions based on the backend enum
const availablePermissions = ref<PermissionOption[]>([
  // Super admin
  { value: '*', label: 'All Permissions', description: 'Full system access', category: 'super' },
  
  // Admin permissions
  { value: 'admin:read', label: 'Admin Read', description: 'Read admin data', category: 'admin' },
  { value: 'admin:write', label: 'Admin Write', description: 'Write admin data', category: 'admin' },
  { value: 'admin:delete', label: 'Admin Delete', description: 'Delete admin data', category: 'admin' },
  { value: 'admin:users', label: 'Admin Users', description: 'Manage users', category: 'admin' },
  { value: 'admin:roles', label: 'Admin Roles', description: 'Manage roles', category: 'admin' },
  { value: 'admin:logs', label: 'Admin Logs', description: 'View system logs', category: 'admin' },
  { value: 'admin:database', label: 'Admin Database', description: 'Access database', category: 'admin' },
  { value: 'admin:health', label: 'Admin Health', description: 'View system health', category: 'admin' },
  { value: 'admin:metrics', label: 'Admin Metrics', description: 'View system metrics', category: 'admin' },
  
  // User permissions
  { value: 'user:read', label: 'User Read', description: 'Read user data', category: 'user' },
  { value: 'user:write', label: 'User Write', description: 'Write user data', category: 'user' },
  { value: 'user:delete', label: 'User Delete', description: 'Delete user data', category: 'user' },
  { value: 'user:profile', label: 'User Profile', description: 'Manage user profile', category: 'user' },
  { value: 'user:create', label: 'User Create', description: 'Create users', category: 'user' },
  
  // System permissions
  { value: 'system:health', label: 'System Health', description: 'View system health', category: 'system' },
  { value: 'system:metrics', label: 'System Metrics', description: 'View system metrics', category: 'system' },
  { value: 'system:logs', label: 'System Logs', description: 'View system logs', category: 'system' },
  { value: 'system:database', label: 'System Database', description: 'Access system database', category: 'system' },
])

const columnDefs = ref([
  {
    headerName: 'ID',
    field: 'id',
    width: 80,
    sortable: true,
    filter: true
  },
  {
    headerName: 'Name',
    field: 'name',
    flex: 1,
    sortable: true,
    filter: true
  },
  {
    headerName: 'Description',
    field: 'description',
    flex: 2,
    cellRenderer: (params: any) => {
      return params.value || 'No description'
    },
    sortable: true,
    filter: true
  },
  {
    headerName: 'Permissions Count',
    field: 'permissions',
    width: 150,
    cellRenderer: (params: any) => {
      return `${params.value?.length || 0} permissions`
    },
    sortable: true
  },
  {
    headerName: 'Created',
    field: 'created_at',
    flex: 1,
    valueFormatter: (params: any) => formatTimestamp(params.value),
    sortable: true,
    filter: true
  },
  {
    headerName: 'Actions',
    flex: 1,
    cellRenderer: (params: any) => {
      return `
        <div style="display: flex; gap: 8px;">
          <button onclick="window.viewRole('${params.data.id}')" style="background: #007bff; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">View</button>
          <button onclick="window.editRole('${params.data.id}')" style="background: #ffc107; color: black; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">Edit</button>
          <button onclick="window.deleteRole('${params.data.id}')" style="background: #dc3545; color: white; border: none; padding: 4px 8px; border-radius: 4px; cursor: pointer; font-size: 12px;">Delete</button>
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

const rowSelection = {
  mode: 'singleRow',
} as any

const creating = ref(false)
const updating = ref(false)
const deleting = ref(false)
const deleteError = ref('')

const fetchRoles = async () => {
  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const query: GetRolesHandlerData["query"] = {
      page: filters.page,
      limit: filters.limit
    }

    if (filters.search) query.search = filters.search

    const response = await getRolesHandler({
      query,
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      console.log(response.data)
      roles.value = response.data.data || []
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
    console.error('Error fetching roles:', error)
  } finally {
    loading.value = false
  }
}

const createRole = async () => {
  creating.value = true
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await createRoleHandler({
      body: {
        name: createForm.name,
        description: createForm.description || null,
        permissions: createForm.permissions
      },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showCreateModal.value = false
      resetCreateForm()
      fetchRoles()
    }
  } catch (error) {
    console.error('Error creating role:', error)
  } finally {
    creating.value = false
  }
}

const updateRole = async () => {
  updating.value = true
  if (!selectedRole.value) return

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const updateData: any = {
      name: editForm.name,
      description: editForm.description || null,
      permissions: editForm.permissions
    }

    const response = await updateRoleHandler({
      path: { id: selectedRole.value.id },
      body: updateData,
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showEditModal.value = false
      resetEditForm()
      fetchRoles()
    }
  } catch (error) {
    console.error('Error updating role:', error)
  } finally {
    updating.value = false
  }
}

const confirmDelete = async () => {
  deleting.value = true
  if (!selectedRole.value) return

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await deleteRoleHandler<false>({
      path: { id: selectedRole.value.id },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      showDeleteModal.value = false
      selectedRole.value = null
      deleteError.value = ''
      fetchRoles()
    } else if (response.error) {
      console.error('Error deleting role:', response.error)
      deleteError.value = response.error.message || 'Failed to delete role'
    }
  } catch (error: any) {
    console.error('Error deleting role:', error)
    deleteError.value = error.message || 'Failed to delete role'
  } finally {
    deleting.value = false
  }
}

const refreshRoles = () => {
  fetchRoles()
}

const onRowClicked = (params: any) => {
  selectedRole.value = params.data
  showDetailModal.value = true
}

// Modal functions
const closeCreateModal = () => {
  showCreateModal.value = false
  resetCreateForm()
}

const closeEditModal = () => {
  showEditModal.value = false
  selectedRole.value = null
}

const closeDeleteModal = () => {
  showDeleteModal.value = false
  selectedRole.value = null
  deleteError.value = ''
}

const closeDetailModal = () => {
  showDetailModal.value = false
  selectedRole.value = null
}

const openEditModal = () => {
  if (selectedRole.value) {
    populateEditForm(selectedRole.value)
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

const viewRole = (roleId: number) => {
  selectedRole.value = roles.value.find(r => r.id === roleId) || null
  showDetailModal.value = true
}

const editRole = (roleId: number) => {
  const role = roles.value.find(r => r.id === roleId)
  if (role) {
    selectedRole.value = role
    populateEditForm(role)
    showEditModal.value = true
  }
}

const deleteRole = (roleId: number) => {
  selectedRole.value = roles.value.find(r => r.id === roleId) || null
  showDeleteModal.value = true
}

const populateEditForm = (role: Role) => {
  editForm.name = role.name
  editForm.description = role.description || ''
  
  // Separate predefined and custom permissions
  const predefinedPermissions = availablePermissions.value.map(p => p.value)
  const currentPermissions = role.permissions || []
  
  editForm.permissions = currentPermissions.filter(p => predefinedPermissions.includes(p))
  editCustomPermissionInput.value = currentPermissions
    .filter(p => !predefinedPermissions.includes(p))
    .join(', ')
}

const resetCreateForm = () => {
  createForm.name = ''
  createForm.description = ''
  createForm.permissions = []
  customPermissionInput.value = ''
}

const resetEditForm = () => {
  editForm.name = ''
  editForm.description = ''
  editForm.permissions = []
  editCustomPermissionInput.value = ''
  selectedRole.value = null
}

const updateCustomPermissions = () => {
  const customPerms = customPermissionInput.value
    .split(',')
    .map(p => p.trim())
    .filter(p => p.length > 0)
  
  // Remove existing custom permissions and add new ones
  const predefinedPermissions = availablePermissions.value.map(p => p.value)
  createForm.permissions = createForm.permissions.filter(p => predefinedPermissions.includes(p))
  createForm.permissions.push(...customPerms)
}

const updateEditCustomPermissions = () => {
  const customPerms = editCustomPermissionInput.value
    .split(',')
    .map(p => p.trim())
    .filter(p => p.length > 0)
  
  // Remove existing custom permissions and add new ones
  const predefinedPermissions = availablePermissions.value.map(p => p.value)
  editForm.permissions = editForm.permissions.filter(p => predefinedPermissions.includes(p))
  editForm.permissions.push(...customPerms)
}

const applyFilters = () => {
  filters.page = 1
  fetchRoles()
}

const onGridReady = (params: any) => {
  gridApi.value = params.api
}

const exportToCsv = () => {
  if (!gridApi.value) return

  const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-')
  const fileName = `roles-${timestamp}.csv`

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

// Global functions for AG Grid action buttons
onMounted(() => {
  fetchRoles()
  
  // Set up global functions for AG Grid action buttons
  ;(window as any).viewRole = viewRole
  ;(window as any).editRole = editRole
  ;(window as any).deleteRole = deleteRole
})
</script>

<style scoped>
.admin-roles {
  padding: 2rem;
}

.roles-header {
  margin-bottom: 2rem;
}

.roles-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.roles-header p {
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
  max-width: 600px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
}

.modal-content.delete-modal {
  max-width: 400px;
}

.modal-content.create-modal,
.modal-content.edit-modal {
  max-width: 700px;
}

.modal-content.detail-modal {
  max-width: 500px;
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

.role-form {
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
.form-group textarea,
.form-group select {
  padding: 0.75rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
}

.permissions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1rem;
  max-height: 300px;
  overflow-y: auto;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 1rem;
  background: #f8f9fa;
}

.permission-item {
  background: white;
  border-radius: 6px;
  padding: 0.75rem;
  border: 1px solid #e0e0e0;
}

.checkbox-label {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
  cursor: pointer;
}

.checkbox-label input[type="checkbox"] {
  width: auto;
  margin: 0 0 0.5rem 0;
}

.permission-name {
  font-weight: 500;
  color: #333;
  font-size: 0.9rem;
}

.permission-description {
  color: #666;
  font-size: 0.8rem;
}

.custom-permissions {
  margin-top: 1rem;
}

.custom-permissions label {
  font-size: 0.9rem;
  color: #666;
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

.role-info {
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

.role-detail-grid {
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

.detail-item.permissions-detail {
  flex-direction: column;
  align-items: stretch;
}

.detail-item label {
  font-weight: 600;
  color: #333;
}

.detail-item span {
  color: #666;
}

.permissions-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  margin-top: 0.5rem;
}

.permission-tag {
  background: #667eea;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 500;
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

@media (max-width: 768px) {
  .admin-roles {
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

  .form-actions {
    flex-direction: column;
  }

  .detail-actions {
    flex-direction: column;
  }

  .permissions-grid {
    grid-template-columns: 1fr;
  }
}
</style> 