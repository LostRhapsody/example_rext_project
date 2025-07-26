<template>
  <div class="admin-logs">
    <div class="logs-header">
      <h1>Request Logs</h1>
      <p>Monitor and analyze API request history</p>
    </div>

    <!-- AG Grid -->
    <div class="grid-container">
      <ag-grid-vue
        :columnDefs="columnDefs"
        :defaultColDef="defaultColDef"
        :rowData="logs"
        :pagination="true"
        :paginationPageSize="filters.limit"
        :rowSelection="rowSelection"
        :animateRows="true"
        :domLayout="'autoHeight'"
        @row-clicked="onRowClicked"
      />
    </div>

    <!-- Log Detail Modal -->
    <div v-if="showLogModal" class="modal-overlay" @click="closeLogModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Log Details</h3>
          <button @click="closeLogModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="log-detail-grid">
            <div class="detail-item">
              <label>ID:</label>
              <span>{{ selectedLog?.id }}</span>
            </div>
            <div class="detail-item">
              <label>Timestamp:</label>
              <span>{{ formatTimestamp(selectedLog?.timestamp || '') }}</span>
            </div>
            <div class="detail-item">
              <label>Method:</label>
              <span class="method-badge">{{ selectedLog?.method }}</span>
            </div>
            <div class="detail-item">
              <label>Path:</label>
              <span>{{ selectedLog?.path }}</span>
            </div>
            <div class="detail-item">
              <label>Status Code:</label>
              <span :class="getStatusClass(selectedLog?.status_code || 0)">{{ selectedLog?.status_code }}</span>
            </div>
            <div class="detail-item">
              <label>Response Time:</label>
              <span>{{ selectedLog?.response_time_ms }}ms</span>
            </div>
            <div class="detail-item">
              <label>User ID:</label>
              <span>{{ selectedLog?.user_id || 'N/A' }}</span>
            </div>
            <div class="detail-item">
              <label>IP Address:</label>
              <span>{{ selectedLog?.ip_address || 'N/A' }}</span>
            </div>
            <div class="detail-item full-width">
              <label>User Agent:</label>
              <span>{{ selectedLog?.user_agent || 'N/A' }}</span>
            </div>
            <div v-if="selectedLog?.request_body" class="detail-item full-width">
              <label>Request Body:</label>
              <pre>{{ selectedLog.request_body }}</pre>
            </div>
            <div v-if="selectedLog?.response_body" class="detail-item full-width">
              <label>Response Body:</label>
              <pre>{{ selectedLog.response_body }}</pre>
            </div>
            <div v-if="selectedLog?.error_message" class="detail-item full-width">
              <label>Error Message:</label>
              <pre class="error-message">{{ selectedLog.error_message }}</pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { getAuditLogsHandler } from '@/bridge/client'
import { AgGridVue } from 'ag-grid-vue3'
import type { ColDef } from "ag-grid-community";

const rowSelection = 'single'

interface LogEntry {
  id: string
  timestamp?: string | null
  method: string
  path: string
  status_code?: number | null
  response_time_ms?: number | null
  user_id?: string | null
  ip_address?: string | null
  user_agent?: string | null
  error_message?: string | null
  request_body?: string | null
  response_body?: string | null
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
const logs = ref<LogEntry[]>([])
const paginationInfo = ref<PaginationInfo>({
  page: 1,
  limit: 100,
  total: 0,
  totalPages: 0,
  start: 0,
  end: 0
})

const filters = reactive({
  page: 1,
  limit: 20,
  method: '',
  status_code: null as number | null,
  user_id: '',
  start_date: '',
  end_date: ''
})

const selectedLog = ref<LogEntry | null>(null)
const showLogModal = ref(false)

const columnDefs = ref<ColDef<LogEntry>[]>([
  {
    headerName: 'Timestamp',
    field: 'timestamp',
    width: 180,
    valueFormatter: (params: any) => formatTimestamp(params.value),
    sortable: true,
    filter: true
  },
  {
    headerName: 'Method',
    field: 'method',
    width: 100,
    cellRenderer: (params: any) => {
      const method = params.value
      const color = method === 'GET' ? '#28a745' :
                   method === 'POST' ? '#007bff' :
                   method === 'PUT' ? '#ffc107' :
                   method === 'DELETE' ? '#dc3545' : '#6c757d'
      return `<span style="color: ${color}; font-weight: bold;">${method}</span>`
    },
    sortable: true,
    filter: true
  },
  {
    headerName: 'Path',
    field: 'path',
    flex: 1,
    sortable: true,
    filter: true
  },
  {
    headerName: 'Status',
    field: 'status_code',
    width: 100,
    cellRenderer: (params: any) => {
      const status = params.value
      const color = status >= 200 && status < 300 ? '#28a745' :
                   status >= 400 && status < 500 ? '#ffc107' :
                   status >= 500 ? '#dc3545' : '#6c757d'
      return `<span style="color: ${color}; font-weight: bold;">${status}</span>`
    },
    sortable: true,
    filter: true
  },
  {
    headerName: 'Response Time',
    field: 'response_time_ms',
    width: 120,
    valueFormatter: (params: any) => params.value ? `${params.value}ms` : 'N/A',
    sortable: true,
    filter: true
  },
  {
    headerName: 'User ID',
    field: 'user_id',
    width: 150,
    valueFormatter: (params: any) => params.value || 'N/A',
    sortable: true,
    filter: true
  },
  {
    headerName: 'IP Address',
    field: 'ip_address',
    width: 130,
    valueFormatter: (params: any) => params.value || 'N/A',
    sortable: true,
    filter: true
  }
])

const defaultColDef = {
  flex: 1,
  resizable: true,
  sortable: true,
  filter: true,
}

const onRowClicked = (event: any) => {
  selectedLog.value = event.data
  showLogModal.value = true
}

const closeLogModal = () => {
  showLogModal.value = false
  selectedLog.value = null
}

const applyFilters = () => {
  filters.page = 1
  fetchLogs()
}

const clearFilters = () => {
  filters.method = ''
  filters.status_code = null
  filters.user_id = ''
  filters.start_date = ''
  filters.end_date = ''
  filters.page = 1
  fetchLogs()
}

const changePage = (newPage: number) => {
  if (newPage >= 1 && newPage <= paginationInfo.value.totalPages) {
    filters.page = newPage
    fetchLogs()
  }
}

const formatTimestamp = (timestamp?: string) => {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return timestamp
  }
}

const getStatusClass = (statusCode?: number) => {
  if (!statusCode) return 'status-unknown'
  if (statusCode >= 200 && statusCode < 300) return 'status-success'
  if (statusCode >= 400 && statusCode < 500) return 'status-client-error'
  if (statusCode >= 500) return 'status-server-error'
  return 'status-unknown'
}

const fetchLogs = async () => {
  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const query: any = {
      page: filters.page,
      limit: filters.limit
    }

    if (filters.method) query.method = filters.method
    if (filters.status_code) query.status_code = filters.status_code
    if (filters.user_id) query.user_id = filters.user_id
    if (filters.start_date) query.start_date = filters.start_date
    if (filters.end_date) query.end_date = filters.end_date

    const response = await getAuditLogsHandler({
      query,
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      logs.value = response.data.data || []
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
    console.error('Error fetching logs:', error)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchLogs()
})
</script>

<style scoped>
.admin-logs {
  padding: 2rem;
}

.logs-header {
  margin-bottom: 2rem;
}

.logs-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.logs-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.filters-section {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  margin-bottom: 2rem;
}

.filters-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
  margin-bottom: 1rem;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.filter-group label {
  font-weight: 500;
  color: #333;
  font-size: 0.9rem;
}

.filter-group select,
.filter-group input {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
}

.filter-actions {
  display: flex;
  gap: 1rem;
  justify-content: flex-end;
}

.clear-btn,
.refresh-btn {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.clear-btn {
  background: #f8f9fa;
  color: #666;
  border: 1px solid #ddd;
}

.clear-btn:hover {
  background: #e9ecef;
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
  padding: 1rem;
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
  max-width: 800px;
  width: 90%;
  max-height: 90vh;
  overflow-y: auto;
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

.log-detail-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.detail-item.full-width {
  grid-column: 1 / -1;
}

.detail-item label {
  font-weight: 600;
  color: #333;
  font-size: 0.9rem;
}

.detail-item span {
  color: #666;
  word-break: break-all;
}

.detail-item pre {
  background: #f8f9fa;
  padding: 0.75rem;
  border-radius: 6px;
  font-size: 0.85rem;
  overflow-x: auto;
  margin: 0;
}

.detail-item pre.error-message {
  background: #fee;
  color: #c33;
  border: 1px solid #fcc;
}

.method-badge {
  background: #667eea;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
  display: inline-block;
  width: fit-content;
}

/* AG Grid custom styles */
:deep(.status-badge) {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

:deep(.status-badge.status-success) {
  background: #d4edda;
  color: #155724;
}

:deep(.status-badge.status-client-error) {
  background: #f8d7da;
  color: #721c24;
}

:deep(.status-badge.status-server-error) {
  background: #f5c6cb;
  color: #721c24;
}

:deep(.status-badge.status-unknown) {
  background: #e2e3e5;
  color: #383d41;
}

@media (max-width: 768px) {
  .admin-logs {
    padding: 1rem;
  }

  .filters-grid {
    grid-template-columns: 1fr;
  }

  .pagination-info {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .log-detail-grid {
    grid-template-columns: 1fr;
  }
}
</style>