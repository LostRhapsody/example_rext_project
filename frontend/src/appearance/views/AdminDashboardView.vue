<template>
  <div class="admin-dashboard">
    <div class="dashboard-header">
      <h1>Admin Dashboard</h1>
      <p>Welcome to the Rext Admin Panel</p>
    </div>

    <!-- Stats Cards -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">👥</div>
        <div class="stat-content">
          <h3>{{ stats.totalUsers || 0 }}</h3>
          <p>Total Users</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">📋</div>
        <div class="stat-content">
          <h3>{{ stats.totalLogs || 0 }}</h3>
          <p>Request Logs</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">🗄️</div>
        <div class="stat-content">
          <h3>{{ stats.totalTables || 0 }}</h3>
          <p>Database Tables</p>
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-icon">💚</div>
        <div class="stat-content">
          <h3>{{ stats.systemStatus || 'Unknown' }}</h3>
          <p>System Status</p>
        </div>
      </div>
    </div>

    <!-- Recent Activity -->
    <div class="dashboard-section">
      <h2>Recent Activity</h2>
      <div class="activity-list">
        <div v-if="loading" class="loading">Loading recent activity...</div>
        <div v-else-if="recentLogs.length === 0" class="no-data">No recent activity</div>
        <div v-else class="activity-items">
          <div
            v-for="log in recentLogs"
            :key="log.id"
            class="activity-item"
          >
            <div class="activity-icon" :class="getStatusClass(log.status_code || 0)">
              {{ getMethodIcon(log.method) }}
            </div>
            <div class="activity-content">
              <div class="activity-title">{{ log.method }} {{ log.path }}</div>
              <div class="activity-meta">
                <span class="status-code" :class="getStatusClass(log.status_code || 0)">
                  {{ log.status_code }}
                </span>
                <span class="timestamp">{{ formatTimestamp(log.timestamp || '') }}</span>
                <span v-if="log.response_time_ms" class="response-time">
                  {{ log.response_time_ms }}ms
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="dashboard-section">
      <h2>Quick Actions</h2>
      <div class="quick-actions">
        <router-link to="/admin/users" class="action-card">
          <div class="action-icon">👥</div>
          <h3>Manage Users</h3>
          <p>View and manage user accounts</p>
        </router-link>

        <router-link to="/admin/logs" class="action-card">
          <div class="action-icon">📋</div>
          <h3>View Logs</h3>
          <p>Browse request history and audit logs</p>
        </router-link>

        <router-link to="/admin/database" class="action-card">
          <div class="action-icon">🗄️</div>
          <h3>Database</h3>
          <p>Inspect database tables and records</p>
        </router-link>

        <router-link to="/admin/health" class="action-card">
          <div class="action-icon">💚</div>
          <h3>System Health</h3>
          <p>Monitor system status and performance</p>
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  getAuditLogsHandler,
  getUsersHandler,
  getDatabaseTablesHandler,
  healthHandler
} from '@/bridge/client'

interface Log {
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

interface User {
  id: string
  email: string
  created_at: string
}

interface DatabaseTable {
  name: string
  record_count: number
}

interface HealthStatus {
  status: string
  timestamp: string
}

interface Stats {
  totalUsers: number
  totalLogs: number
  totalTables: number
  systemStatus: string
}

const loading = ref(false)
const stats = ref<Stats>({
  totalUsers: 0,
  totalLogs: 0,
  totalTables: 0,
  systemStatus: 'Unknown'
})
const recentLogs = ref<Log[]>([])

const fetchDashboardData = async () => {
  loading.value = true

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const headers = {
      'Authorization': `Bearer ${token}`
    }

    // Fetch recent logs
    const logsResponse = await getAuditLogsHandler({
      query: { limit: 5 },
      headers
    })

    if (logsResponse.data) {
      recentLogs.value = logsResponse.data.data || []
      stats.value.totalLogs = logsResponse.data.pagination?.total || 0
    }

    // Fetch user count
    const usersResponse = await getUsersHandler({
      query: { limit: 1 },
      headers
    })

    if (usersResponse.data) {
      stats.value.totalUsers = usersResponse.data.pagination?.total || 0
    }

    // Fetch database tables
    const tablesResponse = await getDatabaseTablesHandler({
      headers
    })

    if (tablesResponse.data) {
      stats.value.totalTables = tablesResponse.data.length || 0
    }

    // Fetch system health
    const healthResponse = await healthHandler({
      headers
    })

    if (healthResponse.data) {
      stats.value.systemStatus = healthResponse.data.status || 'Unknown'
    }

  } catch (error) {
    console.error('Error fetching dashboard data:', error)
  } finally {
    loading.value = false
  }
}

const getMethodIcon = (method: string) => {
  const icons: Record<string, string> = {
    GET: '📖',
    POST: '➕',
    PUT: '✏️',
    DELETE: '🗑️',
    PATCH: '🔧'
  }
  return icons[method] || '❓'
}

const getStatusClass = (status: number) => {
  if (status >= 200 && status < 300) return 'status-success'
  if (status >= 400 && status < 500) return 'status-warning'
  if (status >= 500) return 'status-error'
  return 'status-unknown'
}

const formatTimestamp = (timestamp: string) => {
  return new Date(timestamp).toLocaleString()
}

onMounted(() => {
  fetchDashboardData()
})
</script>

<style scoped>
.admin-dashboard {
  padding: 2rem;
}

.dashboard-header {
  margin-bottom: 2rem;
}

.dashboard-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.dashboard-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 3rem;
}

.stat-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  display: flex;
  align-items: center;
  gap: 1rem;
  transition: transform 0.2s, box-shadow 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
}

.stat-icon {
  font-size: 2.5rem;
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 12px;
  color: white;
}

.stat-content h3 {
  margin: 0 0 0.25rem 0;
  font-size: 2rem;
  font-weight: 700;
  color: #333;
}

.stat-content p {
  margin: 0;
  color: #666;
  font-size: 0.9rem;
  font-weight: 500;
}

.dashboard-section {
  margin-bottom: 3rem;
}

.dashboard-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.activity-list {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
}

.loading, .no-data {
  padding: 2rem;
  text-align: center;
  color: #666;
}

.activity-items {
  max-height: 400px;
  overflow-y: auto;
}

.activity-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.activity-item:hover {
  background-color: #f8f9ff;
}

.activity-item:last-child {
  border-bottom: none;
}

.activity-icon {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  font-size: 1.2rem;
  background: #f0f0f0;
}

.activity-content {
  flex: 1;
}

.activity-title {
  font-weight: 600;
  color: #333;
  margin-bottom: 0.25rem;
}

.activity-meta {
  display: flex;
  gap: 1rem;
  font-size: 0.85rem;
  color: #666;
}

.status-code {
  font-weight: 600;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.75rem;
}

.status-success {
  background: #d4edda;
  color: #155724;
}

.status-warning {
  background: #fff3cd;
  color: #856404;
}

.status-error {
  background: #f8d7da;
  color: #721c24;
}

.status-unknown {
  background: #e2e3e5;
  color: #383d41;
}

.quick-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
}

.action-card {
  background: white;
  border-radius: 12px;
  padding: 2rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  text-decoration: none;
  color: inherit;
  transition: transform 0.2s, box-shadow 0.2s;
  text-align: center;
}

.action-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
}

.action-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.action-card h3 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 1.25rem;
  font-weight: 600;
}

.action-card p {
  margin: 0;
  color: #666;
  font-size: 0.9rem;
  line-height: 1.4;
}

@media (max-width: 768px) {
  .admin-dashboard {
    padding: 1rem;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .quick-actions {
    grid-template-columns: 1fr;
  }

  .activity-meta {
    flex-direction: column;
    gap: 0.5rem;
  }
}
</style>