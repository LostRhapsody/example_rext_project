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
          <p>{{ uptime }}</p>
        </div>
      </div>

      <div class="health-card">
        <div class="health-icon">💾</div>
        <div class="health-content">
          <h3>Database</h3>
          <p :class="getStatusClass(dbStatus)">{{ dbStatus }}</p>
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
              <span class="metric-label">Memory Usage:</span>
              <span class="metric-value">{{ metrics.memoryUsage }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">CPU Usage:</span>
              <span class="metric-value">{{ metrics.cpuUsage }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Disk Usage:</span>
              <span class="metric-value">{{ metrics.diskUsage }}</span>
            </div>
            <div class="metric-item">
              <span class="metric-label">Network I/O:</span>
              <span class="metric-value">{{ metrics.networkIO }}</span>
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
      <h2>System Information</h2>
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
            <span class="info-value" :class="getStatusClass(systemInfo.databaseStatus)">
              {{ systemInfo.databaseStatus }}
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

interface HealthStatus {
  status: string
  timestamp: string
}

interface Metrics {
  totalRequests: number
  successRate: number
  avgResponseTime: number
  errorRate: number
  activeUsers: number
  totalUsers: number
  adminUsers: number
  newUsers24h: number
  memoryUsage: string
  cpuUsage: string
  diskUsage: string
  networkIO: string
}

interface SystemInfo {
  version: string
  environment: string
  startTime: string
  databaseType: string
  databaseTables: number
  databaseStatus: string
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
const healthStatus = ref<HealthStatus>({
  status: 'Unknown',
  timestamp: ''
})
const metrics = ref<Metrics>({
  totalRequests: 0,
  successRate: 0,
  avgResponseTime: 0,
  errorRate: 0,
  activeUsers: 0,
  totalUsers: 0,
  adminUsers: 0,
  newUsers24h: 0,
  memoryUsage: 'Unknown',
  cpuUsage: 'Unknown',
  diskUsage: 'Unknown',
  networkIO: 'Unknown'
})
const systemInfo = ref<SystemInfo>({
  version: '1.0.0',
  environment: 'development',
  startTime: '',
  databaseType: 'SQLite',
  databaseTables: 0,
  databaseStatus: 'Unknown',
  host: 'localhost',
  port: 3000,
  protocol: 'HTTP'
})
const recentErrors = ref<ErrorLog[]>([])
const uptime = ref('Unknown')
const dbStatus = ref('Unknown')

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
      timestamp: new Date().toISOString()
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
        newUsers24h: 0, // Would need additional tracking
        memoryUsage: 'N/A', // Would need system monitoring
        cpuUsage: 'N/A', // Would need system monitoring
        diskUsage: 'N/A', // Would need system monitoring
        networkIO: 'N/A' // Would need system monitoring
      }
    }

    // Fetch user statistics
    const usersResponse = await getUsersHandler({
      query: { limit: 1 },
      headers
    })

    if (usersResponse.data) {
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
      systemInfo.value.databaseStatus = 'Connected'
    }

    // Set start time (approximate)
    systemInfo.value.startTime = new Date(Date.now() - 60000).toISOString() // 1 minute ago as approximation

  } catch (error) {
    console.error('Error fetching system info:', error)
    systemInfo.value.databaseStatus = 'Error'
  }
}

const refreshMetrics = () => {
  fetchHealthStatus()
  fetchMetrics()
  fetchSystemInfo()
}

// Update dbStatus when system info is fetched
const updateDbStatus = () => {
  dbStatus.value = systemInfo.value.databaseStatus
}

const getStatusClass = (status: string) => {
  const statusLower = status.toLowerCase()
  if (statusLower === 'healthy' || statusLower === 'ok' || statusLower === 'connected') return 'status-success'
  if (statusLower === 'warning' || statusLower === 'degraded') return 'status-warning'
  if (statusLower === 'error' || statusLower === 'down' || statusLower === 'unhealthy') return 'status-error'
  return 'status-unknown'
}

const getStatusIcon = (status: string) => {
  const statusLower = status.toLowerCase()
  if (statusLower === 'healthy' || statusLower === 'ok') return '💚'
  if (statusLower === 'warning' || statusLower === 'degraded') return '⚠️'
  if (statusLower === 'error' || statusLower === 'down' || statusLower === 'unhealthy') return '🔴'
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

const updateUptime = () => {
  if (systemInfo.value.startTime) {
    try {
      const startTime = new Date(systemInfo.value.startTime).getTime()
      const now = Date.now()
      const diff = now - startTime
      
      const days = Math.floor(diff / (1000 * 60 * 60 * 24))
      const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
      const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
      
      if (days > 0) {
        uptime.value = `${days}d ${hours}h ${minutes}m`
      } else if (hours > 0) {
        uptime.value = `${hours}h ${minutes}m`
      } else {
        uptime.value = `${minutes}m`
      }
    } catch {
      uptime.value = 'Unknown'
    }
  }
}

onMounted(() => {
  refreshMetrics()
  updateDbStatus()
  
  // Update uptime every minute
  updateUptime()
  refreshInterval = setInterval(() => {
    updateUptime()
    updateDbStatus()
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