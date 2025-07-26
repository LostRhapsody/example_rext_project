<template>
  <div class="admin-health">
    <div class="health-header">
      <h1>System Health</h1>
      <p>Monitor system status and performance</p>
    </div>

    <!-- Health Status Cards -->
    <div class="health-grid">
      <div class="health-card">
        <div class="health-icon" :class="getStatusClass(healthStatus.status)">
          {{ getStatusIcon(healthStatus.status) }}
        </div>
        <div class="health-content">
          <h3>System Status</h3>
          <p class="status-text" :class="getStatusClass(healthStatus.status)">
            {{ healthStatus.status || 'Unknown' }}
          </p>
        </div>
      </div>

      <div class="health-card">
        <div class="health-icon">🕐</div>
        <div class="health-content">
          <h3>Last Check</h3>
          <p>{{ formatTimestamp(healthStatus.timestamp) }}</p>
        </div>
      </div>

      <div class="health-card">
        <div class="health-icon">📊</div>
        <div class="health-content">
          <h3>Uptime</h3>
          <p>{{ healthStatus.uptime || 'Unknown' }}</p>
        </div>
      </div>

      <div class="health-card">
        <div class="health-icon">💾</div>
        <div class="health-content">
          <h3>Database</h3>
          <p :class="getStatusClass(healthStatus.database_status)">
            {{ healthStatus.database_status || 'Unknown' }}
          </p>
        </div>
      </div>
    </div>

    <!-- Performance Metrics -->
    <div class="metrics-section">
      <h2>Performance Metrics</h2>
      <div class="metrics-grid">
        <div class="metric-card">
          <div class="metric-header">
            <h3>Request Statistics</h3>
            <button @click="refreshMetrics" class="refresh-btn">
              <span>🔄</span> Refresh
            </button>
          </div>
          <div class="metric-content">
            <div class="metric-item">
              <span class="metric-label">Total Requests:</span>
              <span class="metric-value">{{ metrics.totalRequests }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Success Rate:</span>
              <span class="metric-value">{{ metrics.successRate }}%</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Average Response Time:</span>
              <span class="metric-value">{{ metrics.avgResponseTime }}ms</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Error Rate:</span>
              <span class="metric-value error">{{ metrics.errorRate }}%</span>
            </div>
          </div>
        </div>

        <div class="metric-card">
          <div class="metric-header">
            <h3>User Activity</h3>
          </div>
          <div class="metric-content">
            <div class="metric-item">
              <span class="metric-label">Active Users:</span>
              <span class="metric-value">{{ metrics.activeUsers }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Total Users:</span>
              <span class="metric-value">{{ metrics.totalUsers }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Admin Users:</span>
              <span class="metric-value">{{ metrics.adminUsers }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">New Users (24h):</span>
              <span class="metric-value">{{ metrics.newUsers24h }}</span>
            </div>
          </div>
        </div>

        <div class="metric-card">
          <div class="metric-header">
            <h3>System Resources</h3>
          </div>
          <div class="metric-content">
            <div class="metric-item">
              <span class="metric-label">CPU Usage:</span>
              <span class="metric-value" :class="getUsageClass(healthStatus.cpu_usage)">
                {{ healthStatus.cpu_usage?.toFixed(1) || '0.0' }}%
              </span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Memory Usage:</span>
              <span class="metric-value" :class="getUsageClass(healthStatus.memory_usage)">
                {{ healthStatus.memory_usage?.toFixed(1) || '0.0' }}%
              </span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Memory Total:</span>
              <span class="metric-value">{{ healthStatus.memory_total || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Memory Used:</span>
              <span class="metric-value">{{ healthStatus.memory_used || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Memory Available:</span>
              <span class="metric-value">{{ healthStatus.memory_available || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Disk Usage:</span>
              <span class="metric-value" :class="getUsageClass(healthStatus.disk_usage)">
                {{ healthStatus.disk_usage?.toFixed(1) || '0.0' }}%
              </span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Disk Total:</span>
              <span class="metric-value">{{ healthStatus.disk_total || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Disk Used:</span>
              <span class="metric-value">{{ healthStatus.disk_used || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Disk Available:</span>
              <span class="metric-value">{{ healthStatus.disk_available || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Network Sent:</span>
              <span class="metric-value">{{ healthStatus.network_bytes_sent || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Network Received:</span>
              <span class="metric-value">{{ healthStatus.network_bytes_received || 'Unknown' }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Process Count:</span>
              <span class="metric-value">{{ healthStatus.process_count || 0 }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">DB Connections:</span>
              <span class="metric-value">{{ healthStatus.database_connections || 'Unknown' }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Recent Errors -->
    <div class="errors-section">
      <h2>Recent Errors</h2>
      <div class="errors-list">
        <div v-if="loading" class="loading">Loading recent errors...</div>
        <div v-else-if="recentErrors.length === 0" class="no-errors">
          <div class="no-errors-icon">✅</div>
          <p>No recent errors found</p>
        </div>
        <div v-else class="error-items">
          <div
            v-for="error in recentErrors"
            :key="error.id"
            class="error-item"
          >
            <div class="error-header">
              <span class="error-status">{{ error.status_code }}</span>
              <span class="error-method">{{ error.method }}</span>
              <span class="error-path">{{ error.path }}</span>
              <span class="error-time">{{ formatTimestamp(error.timestamp) }}</span>
            </div>
            <div v-if="error.error_message" class="error-message">
              {{ error.error_message }}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- System Information -->
    <div class="system-info-section">
      <h2>System Information - Not yet implemented</h2>
      <div class="system-info-grid">
        <div class="info-card">
          <h3>Application</h3>
          <div class="info-item">
            <span class="info-label">Version:</span>
            <span class="info-value">{{ systemInfo.version }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Environment:</span>
            <span class="info-value">{{ systemInfo.environment }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Start Time:</span>
            <span class="info-value">{{ formatTimestamp(systemInfo.startTime) }}</span>
          </div>
        </div>

        <div class="info-card">
          <h3>Database</h3>
          <div class="info-item">
            <span class="info-label">Type:</span>
            <span class="info-value">{{ systemInfo.databaseType }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Tables:</span>
            <span class="info-value">{{ systemInfo.databaseTables }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Connection:</span>
            <span class="info-value" :class="getStatusClass(healthStatus.database_status)">
              {{ healthStatus.database_status || 'Unknown' }}
            </span>
          </div>
        </div>

        <div class="info-card">
          <h3>Server</h3>
          <div class="info-item">
            <span class="info-label">Host:</span>
            <span class="info-value">{{ systemInfo.host }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Port:</span>
            <span class="info-value">{{ systemInfo.port }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Protocol:</span>
            <span class="info-value">{{ systemInfo.protocol }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import {
  healthHandler,
  getAuditLogsHandler,
  getUsersHandler,
  getDatabaseTablesHandler
} from '@/bridge/client'
import type { HealthResponse } from '@/bridge/client/types.gen'

interface Metrics {
  totalRequests: number
  successRate: number
  avgResponseTime: number
  errorRate: number
  activeUsers: number
  totalUsers: number
  adminUsers: number
  newUsers24h: number
}

interface SystemInfo {
  version: string
  environment: string
  startTime: string
  databaseType: string
  databaseTables: number
  host: string
  port: number
  protocol: string
}

interface ErrorLog {
  id: string
  timestamp?: string | null
  method: string
  path: string
  status_code?: number | null
  error_message?: string | null
  ip_address?: string | null
  user_agent?: string | null
  user_id?: string | null
  request_body?: string | null
  response_body?: string | null
  response_time_ms?: number | null
}

const loading = ref(false)
const healthStatus = ref<HealthResponse>({
  status: 'Unknown',
  timestamp: '',
  uptime: '',
  cpu_usage: 0,
  memory_usage: 0,
  memory_total: '',
  memory_used: '',
  memory_available: '',
  disk_usage: 0,
  disk_total: '',
  disk_used: '',
  disk_available: '',
  network_bytes_sent: '',
  network_bytes_received: '',
  process_count: 0,
  database_connections: null,
  database_status: 'Unknown'
})
const metrics = ref<Metrics>({
  totalRequests: 0,
  successRate: 0,
  avgResponseTime: 0,
  errorRate: 0,
  activeUsers: 0,
  totalUsers: 0,
  adminUsers: 0,
  newUsers24h: 0
})
const systemInfo = ref<SystemInfo>({
  version: 'unknown',
  environment: 'unknown',
  startTime: '',
  databaseType: 'unknown',
  databaseTables: 0,
  host: 'unknown',
  port: 0,
  protocol: 'unknown'
})
const recentErrors = ref<ErrorLog[]>([])

let refreshInterval: NodeJS.Timeout | null = null

const fetchHealthStatus = async () => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const response = await healthHandler({
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (response.data) {
      healthStatus.value = response.data
    }
  } catch (error) {
    console.error('Error fetching health status:', error)
    healthStatus.value = {
      status: 'Error',
      timestamp: new Date().toISOString(),
      uptime: 'Unknown',
      cpu_usage: 0,
      memory_usage: 0,
      memory_total: 'Unknown',
      memory_used: 'Unknown',
      memory_available: 'Unknown',
      disk_usage: 0,
      disk_total: 'Unknown',
      disk_used: 'Unknown',
      disk_available: 'Unknown',
      network_bytes_sent: 'Unknown',
      network_bytes_received: 'Unknown',
      process_count: 0,
      database_connections: null,
      database_status: 'Error'
    }
  }
}

const fetchMetrics = async () => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    const headers = {
      'Authorization': `Bearer ${token}`
    }

    // Fetch logs for metrics calculation
    const logsResponse = await getAuditLogsHandler({
      query: { limit: 1000 },
      headers
    })

    if (logsResponse.data) {
      const logs = logsResponse.data.data || []

      // Calculate metrics from logs
      const totalRequests = logs.length
      const successfulRequests = logs.filter((log: any) => log.status_code >= 200 && log.status_code < 300).length
      const errorRequests = logs.filter((log: any) => log.status_code >= 400).length
      const responseTimes = logs.map((log: any) => log.response_time_ms || 0).filter((time: number) => time > 0)

      metrics.value = {
        totalRequests,
        successRate: totalRequests > 0 ? Math.round((successfulRequests / totalRequests) * 100) : 0,
        avgResponseTime: responseTimes.length > 0 ? Math.round(responseTimes.reduce((a: number, b: number) => a + b, 0) / responseTimes.length) : 0,
        errorRate: totalRequests > 0 ? Math.round((errorRequests / totalRequests) * 100) : 0,
        activeUsers: 0, // Would need additional tracking
        totalUsers: 0, // Will be fetched separately
        adminUsers: 0, // Will be fetched separately
        newUsers24h: 0 // Would need additional tracking
      }
    }

    // Fetch user statistics
    const usersResponse = await getUsersHandler({
      headers
    })

    if (usersResponse.data) {
      console.log('usersResponse.data', usersResponse.data)
      const totalUsers = usersResponse.data.pagination?.total || 0
      const adminUsers = usersResponse.data.data?.filter((user: any) => user.is_admin).length || 0

      metrics.value.totalUsers = totalUsers
      metrics.value.adminUsers = adminUsers
    }

    // Fetch recent errors
    const errorsResponse = await getAuditLogsHandler({
      query: { limit: 10, status_code: 500 },
      headers
    })

    if (errorsResponse.data) {
      recentErrors.value = errorsResponse.data.data || []
    }

  } catch (error) {
    console.error('Error fetching metrics:', error)
  }
}

const fetchSystemInfo = async () => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) return

    // Fetch database tables
    const tablesResponse = await getDatabaseTablesHandler({
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (tablesResponse.data) {
      systemInfo.value.databaseTables = tablesResponse.data.length || 0
    }

    // Set start time (approximate)
    systemInfo.value.startTime = new Date(Date.now()).toISOString()

  } catch (error) {
    console.error('Error fetching system info:', error)
  }
}

const refreshMetrics = () => {
  fetchHealthStatus()
  fetchMetrics()
  fetchSystemInfo()
}

const getStatusClass = (status: string) => {
  const statusLower = status.toLowerCase()
  if (statusLower === 'healthy' || statusLower === 'ok' || statusLower === 'connected') return 'status-success'
  if (statusLower === 'warning' || statusLower === 'degraded') return 'status-warning'
  if (statusLower === 'error' || statusLower === 'down' || statusLower === 'unhealthy' || statusLower === 'critical') return 'status-error'
  return 'status-unknown'
}

const getUsageClass = (usage: number) => {
  if (usage > 90) return 'status-error'
  if (usage > 80) return 'status-warning'
  if (usage > 70) return 'status-warning'
  return 'status-success'
}

const getStatusIcon = (status: string) => {
  const statusLower = status.toLowerCase()
  if (statusLower === 'healthy' || statusLower === 'ok') return '💚'
  if (statusLower === 'warning' || statusLower === 'degraded') return '⚠️'
  if (statusLower === 'error' || statusLower === 'down' || statusLower === 'unhealthy' || statusLower === 'critical') return '🔴'
  return '❓'
}

const formatTimestamp = (timestamp?: string | null) => {
  if (!timestamp) return 'Unknown'
  try {
    return new Date(timestamp).toLocaleString()
  } catch {
    return timestamp
  }
}

onMounted(() => {
  refreshMetrics()

  // Update metrics every minute
  refreshInterval = setInterval(() => {
    refreshMetrics()
  }, 60000)
})

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval)
  }
})
</script>

<style scoped>
.admin-health {
  padding: 2rem;
}

.health-header {
  margin-bottom: 2rem;
}

.health-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.health-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.health-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 3rem;
}

.health-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  display: flex;
  align-items: center;
  gap: 1rem;
  transition: transform 0.2s, box-shadow 0.2s;
}

.health-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 16px rgba(0,0,0,0.15);
}

.health-icon {
  font-size: 2.5rem;
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
}

.health-content h3 {
  margin: 0 0 0.25rem 0;
  color: #333;
  font-size: 1rem;
  font-weight: 600;
}

.health-content p {
  margin: 0;
  color: #666;
  font-size: 0.9rem;
}

.status-text {
  font-weight: 600;
  font-size: 1.1rem;
}

.metrics-section {
  margin-bottom: 3rem;
}

.metrics-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.metric-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.metric-header h3 {
  margin: 0;
  color: #333;
  font-size: 1.1rem;
  font-weight: 600;
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

.metric-content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}

.metric-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.metric-item:last-child {
  border-bottom: none;
}

.metric-label {
  color: #666;
  font-size: 0.9rem;
}

.metric-value {
  font-weight: 600;
  color: #333;
  font-size: 0.9rem;
}

.metric-value.error {
  color: #dc3545;
}

.errors-section {
  margin-bottom: 3rem;
}

.errors-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.errors-list {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  overflow: hidden;
}

.loading,
.no-errors {
  padding: 2rem;
  text-align: center;
  color: #666;
}

.no-errors-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
}

.error-items {
  max-height: 400px;
  overflow-y: auto;
}

.error-item {
  padding: 1rem 1.5rem;
  border-bottom: 1px solid #f0f0f0;
  transition: background-color 0.2s;
}

.error-item:hover {
  background-color: #f8f9ff;
}

.error-item:last-child {
  border-bottom: none;
}

.error-header {
  display: flex;
  gap: 1rem;
  align-items: center;
  margin-bottom: 0.5rem;
  font-size: 0.9rem;
}

.error-status {
  background: #dc3545;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.8rem;
}

.error-method {
  background: #6c757d;
  color: white;
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-weight: 600;
  font-size: 0.8rem;
}

.error-path {
  color: #333;
  font-weight: 500;
  flex: 1;
}

.error-time {
  color: #666;
  font-size: 0.8rem;
}

.error-message {
  color: #dc3545;
  font-size: 0.85rem;
  background: #fee;
  padding: 0.5rem;
  border-radius: 4px;
  border-left: 3px solid #dc3545;
}

.system-info-section {
  margin-bottom: 2rem;
}

.system-info-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.system-info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1.5rem;
}

.info-card {
  background: white;
  border-radius: 12px;
  padding: 1.5rem;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.info-card h3 {
  margin: 0 0 1rem 0;
  color: #333;
  font-size: 1.1rem;
  font-weight: 600;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.5rem 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  color: #666;
  font-size: 0.9rem;
}

.info-value {
  font-weight: 600;
  color: #333;
  font-size: 0.9rem;
}

/* Status classes */
.status-success {
  color: #28a745;
}

.status-warning {
  color: #ffc107;
}

.status-error {
  color: #dc3545;
}

.status-unknown {
  color: #6c757d;
}

@media (max-width: 768px) {
  .admin-health {
    padding: 1rem;
  }

  .health-grid {
    grid-template-columns: 1fr;
  }

  .metrics-grid {
    grid-template-columns: 1fr;
  }

  .system-info-grid {
    grid-template-columns: 1fr;
  }

  .error-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .metric-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }

  .info-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.25rem;
  }
}
</style>