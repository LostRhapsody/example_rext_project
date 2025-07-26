<template>
  <div class="admin-database">
    <div class="database-header">
      <h1>Database Browser</h1>
      <p>Inspect database tables and records</p>
    </div>

    <!-- Tables Overview -->
    <div class="tables-section">
      <h2>Database Tables</h2>
      <div class="tables-grid">
        <div
          v-for="table in tables"
          :key="table.name"
          class="table-card"
          :class="{ active: selectedTable === table.name }"
          @click="selectTable(table.name)"
        >
          <div class="table-icon">🗄️</div>
          <div class="table-info">
            <h3>{{ table.name }}</h3>
            <p>{{ table.record_count }} records</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Table Records -->
    <div v-if="selectedTable" class="records-section">

      <!-- AG Grid for Records -->
      <div class="grid-container">
        <ag-grid-vue
          v-if="columns.length > 0"
          :columnDefs="columnDefs"
          :rowData="records"
          :defaultColDef="defaultColDef"
          :pagination="true"
          :paginationPageSize="20"
          :rowSelection="rowSelection"
          :animateRows="true"
          :domLayout="'autoHeight'"
          @row-clicked="onRowClicked"
        />
        <div v-else class="no-records">
          <p>No records found in this table</p>
        </div>
      </div>
    </div>

    <!-- Record Detail Modal -->
    <div v-if="showRecordModal" class="modal-overlay" @click="closeRecordModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>Record Details</h3>
          <button @click="closeRecordModal" class="close-btn">&times;</button>
        </div>
        <div class="modal-body">
          <div class="record-detail-grid">
            <div
              v-for="(value, key) in selectedRecord"
              :key="key"
              class="detail-item"
            >
              <label>{{ key }}:</label>
              <span>{{ formatValue(value) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- No Table Selected State -->
    <div v-if="!selectedTable" class="no-table-selected">
      <div class="empty-state">
        <div class="empty-icon">🗄️</div>
        <h3>Select a Table</h3>
        <p>Choose a table from the list above to view its records</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { AgGridVue } from 'ag-grid-vue3'
import { getDatabaseTablesHandler, getTableRecordsHandler } from '@/bridge/client'

interface DatabaseTable {
  name: string
  record_count: number
}

interface TableRecord {
  columns: string[]
  records: unknown[][]
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
const tables = ref<DatabaseTable[]>([])
const selectedTable = ref('')
const columns = ref<string[]>([])
const records = ref<{ [key: string]: unknown }[]>([])
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
  limit: 9999
})

const showRecordModal = ref(false)
const selectedRecord = ref<{ [key: string]: unknown } | null>(null)

// Column configuration interface
interface ColumnConfig {
  hide?: boolean
  flex?: number
  width?: number
  sortable?: boolean
  filter?: boolean
  valueFormatter?: (params: any) => string
  cellRenderer?: string
  cellRendererParams?: any
  [key: string]: any // Allow additional AG Grid properties
}

// Column configuration overrides for specific tables
const columnOverrides: Record<string, Record<string, ColumnConfig>> = {
  // Example: Hide specific columns for certain tables
  'Jobs': {
    'id': { hide: true },
    'attempts': { hide: true },
    'max_attempts': { hide: true },
    'lock_at': { hide: true },
    'lock_by': { hide: true },
    'last_error': { hide: true },
    'job': { hide: true}
  },
  // Add more table-specific configurations as needed
  'audit_logs': {
    'id': { hide: true },
    'user_agent': { hide: true },
    'request_body': { hide: true },
    'response_body': { hide: true },
    'error_message': { hide: true }
  }
}

/*
 * USAGE EXAMPLES:
 *
 * 1. Hide a column for a specific table:
 *    hideColumn('users', 'password_hash')
 *
 * 2. Show a previously hidden column:
 *    showColumn('users', 'password_hash')
 *
 * 3. Add custom formatting for a column:
 *    addColumnOverride('users', 'email', {
 *      valueFormatter: (params) => params.value?.toLowerCase() || ''
 *    })
 *
 * 4. Set custom width for a column:
 *    addColumnOverride('audit_logs', 'details', { width: 400 })
 *
 * 5. Disable sorting for a column:
 *    addColumnOverride('users', 'id', { sortable: false })
 *
 * 6. Remove a custom override:
 *    removeColumnOverride('users', 'email')
 */

// Helper function to get column configuration for a specific table and field
const getColumnConfig = (tableName: string, fieldName: string): ColumnConfig => {
  const tableConfig = columnOverrides[tableName]
  return tableConfig?.[fieldName] || {}
}

// Utility functions for dynamic column management
const addColumnOverride = (tableName: string, fieldName: string, config: ColumnConfig) => {
  if (!columnOverrides[tableName]) {
    columnOverrides[tableName] = {}
  }
  columnOverrides[tableName][fieldName] = config
}

const removeColumnOverride = (tableName: string, fieldName: string) => {
  if (columnOverrides[tableName]) {
    delete columnOverrides[tableName][fieldName]
  }
}

const hideColumn = (tableName: string, fieldName: string) => {
  addColumnOverride(tableName, fieldName, { hide: true })
}

const showColumn = (tableName: string, fieldName: string) => {
  addColumnOverride(tableName, fieldName, { hide: false })
}

const columnDefs = computed(() => {
  return columns.value
    .map(column => {
      const config = getColumnConfig(selectedTable.value, column)

      // Skip columns that are configured to be hidden
      if (config.hide) {
        return null
      }

      return {
        headerName: column,
        field: column,
        flex: config.flex || 1,
        width: config.width,
        sortable: config.sortable !== false,
        filter: config.filter !== false,
        valueFormatter: config.valueFormatter || ((params: any) => formatValue(params.value)),
        cellRenderer: config.cellRenderer,
        cellRendererParams: config.cellRendererParams,
        ...config // Spread any additional AG Grid properties
      }
    })
    .filter((col): col is NonNullable<typeof col> => col !== null) // Remove null entries (hidden columns)
})

const defaultColDef = {
  resizable: true,
  sortable: true,
  filter: true,
  floatingFilter: true
}

const rowSelection = {
  mode: 'singleRow',
} as any; // AG Grid type definitions are overly strict

const fetchTables = async () => {
  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await getDatabaseTablesHandler({
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      tables.value = response.data || []
    }
  } catch (error) {
    console.error('Error fetching tables:', error)
  } finally {
    loading.value = false
  }
}

const fetchRecords = async () => {
  if (!selectedTable.value) return

  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await getTableRecordsHandler({
      path: { table_name: selectedTable.value },
      query: {
        page: filters.page,
        limit: filters.limit
      },
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      const data = response.data
      columns.value = data.columns || []

      // Transform records from arrays to objects for AG Grid
      const rawRecords = data.records || []
      records.value = rawRecords.map((recordArray: unknown[]) => {
        const recordObject: { [key: string]: unknown } = {}
        columns.value.forEach((column, index) => {
          recordObject[column] = recordArray[index]
        })
        return recordObject
      })

      // Calculate pagination info (approximate since we don't have total from API)
      paginationInfo.value = {
        page: filters.page,
        limit: filters.limit,
        total: records.value.length,
        totalPages: Math.ceil(records.value.length / filters.limit),
        start: (filters.page - 1) * filters.limit + 1,
        end: Math.min(filters.page * filters.limit, records.value.length)
      }
    }
  } catch (error) {
    console.error('Error fetching records:', error)
  } finally {
    loading.value = false
  }
}

const selectTable = (tableName: string) => {
  selectedTable.value = tableName
  filters.page = 1
  fetchRecords()
}

const refreshRecords = () => {
  fetchRecords()
}

const previousPage = () => {
  if (filters.page > 1) {
    filters.page--
    fetchRecords()
  }
}

const nextPage = () => {
  if (filters.page < paginationInfo.value.totalPages) {
    filters.page++
    fetchRecords()
  }
}

const onGridReady = (params: any) => {
  // gridApi.value = params.api // This line is removed as per the new_code
}

const onRowClicked = (event: any) => {
  const recordData = event.data
  selectedRecord.value = recordData
  showRecordModal.value = true
}

const closeRecordModal = () => {
  showRecordModal.value = false
  selectedRecord.value = null
}

const formatValue = (value: unknown) => {
  if (value === null || value === undefined) return 'NULL'
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

const formatTimestamp = (timestamp?: string) => {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return timestamp
  }
}

onMounted(() => {
  fetchTables()
})
</script>

<style scoped>
.admin-database {
  padding: 2rem;
}

.database-header {
  margin-bottom: 2rem;
}

.database-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.database-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.tables-section {
  margin-bottom: 3rem;
}

.tables-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.tables-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 1rem;
}

.table-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  cursor: pointer;
  transition: all 0.2s;
  border: 2px solid transparent;
}

.table-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
}

.table-card.active {
  border-color: #667eea;
  background: #f8f9ff;
}

.table-card {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.table-icon {
  font-size: 2rem;
  width: 50px;
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f8f9fa;
  border-radius: 8px;
}

.table-info h3 {
  margin: 0 0 0.25rem 0;
  color: #333;
  font-size: 1.1rem;
  font-weight: 600;
}

.table-info p {
  margin: 0;
  color: #666;
  font-size: 0.9rem;
}

.records-section {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
}

.records-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1.5rem;
  border-bottom: 1px solid #e0e0e0;
}

.records-header h2 {
  margin: 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.records-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.page-size-select {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 0.9rem;
}

.refresh-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.2s;
}

.refresh-btn:hover {
  background: #5a6fd8;
}

.grid-container {
  padding: 1.5rem;
}

.ag-theme-alpine {
  height: 600px;
  width: 100%;
}

.no-records {
  padding: 3rem;
  text-align: center;
  color: #666;
}

.pagination-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  border-top: 1px solid #e0e0e0;
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

.no-table-selected {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.empty-state {
  text-align: center;
  color: #666;
}

.empty-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.empty-state h3 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 1.5rem;
}

.empty-state p {
  margin: 0;
  font-size: 1rem;
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

.record-detail-grid {
  display: grid;
  gap: 1rem;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 0.75rem;
  background: #f8f9fa;
  border-radius: 6px;
  gap: 1rem;
}

.detail-item label {
  font-weight: 600;
  color: #333;
  min-width: 120px;
  flex-shrink: 0;
}

.detail-item span {
  color: #666;
  word-break: break-all;
  text-align: right;
  flex: 1;
}

@media (max-width: 768px) {
  .admin-database {
    padding: 1rem;
  }

  .tables-grid {
    grid-template-columns: 1fr;
  }

  .records-header {
    flex-direction: column;
    gap: 1rem;
    align-items: stretch;
  }

  .records-actions {
    justify-content: center;
  }

  .pagination-info {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .detail-item {
    flex-direction: column;
    align-items: stretch;
    gap: 0.5rem;
  }

  .detail-item label {
    min-width: auto;
  }

  .detail-item span {
    text-align: left;
  }
}
</style>