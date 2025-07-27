<template>
  <div class="admin-realtime-monitor">
    <div class="monitor-header">
      <h1>Real-time Monitor</h1>
      <p>Live monitoring of system requests and performance</p>
    </div>

    <!-- Connection Status -->
    <div class="connection-status">
      <div class="status-indicator" :class="{ connected: isConnected, disconnected: !isConnected }">
        <span class="status-dot"></span>
        <span class="status-text">{{ connectionStatus }}</span>
      </div>
      <div class="connection-controls">
        <button
          v-if="!isConnected"
          @click="connect"
          class="connect-btn"
          :disabled="connecting"
        >
          <span v-if="connecting">Connecting...</span>
          <span v-else>🔌 Connect</span>
        </button>
        <button
          v-else
          @click="disconnect"
          class="disconnect-btn"
        >
          🔌 Disconnect
        </button>
      </div>
    </div>

    <!-- Live Metrics -->
    <div class="metrics-section">
      <h2>Live Metrics</h2>
      <div class="metrics-grid">
        <div class="metric-card">
          <div class="metric-icon">📊</div>
          <div class="metric-content">
            <h3>Total Requests</h3>
            <p class="metric-value">{{ metrics.totalRequests }}</p>
          </div>
        </div>
        <div class="metric-card">
          <div class="metric-icon">✅</div>
          <div class="metric-content">
            <h3>Success Rate</h3>
            <p class="metric-value">{{ (metrics.successRate * 100).toFixed(1) }}%</p>
          </div>
        </div>
        <div class="metric-card">
          <div class="metric-icon">⚡</div>
          <div class="metric-content">
            <h3>Avg Response Time</h3>
            <p class="metric-value">{{ metrics.avgResponseTime.toFixed(0) }}ms</p>
          </div>
        </div>
        <div class="metric-card">
          <div class="metric-icon">❌</div>
          <div class="metric-content">
            <h3>Error Rate</h3>
            <p class="metric-value">{{ (metrics.errorRate * 100).toFixed(1) }}%</p>
          </div>
        </div>
        <div class="metric-card">
          <div class="metric-icon">👥</div>
          <div class="metric-content">
            <h3>Active Connections</h3>
            <p class="metric-value">{{ metrics.activeConnections }}</p>
          </div>
        </div>
      </div>
    </div>

    <!-- Live Request Stream -->
    <div class="requests-section">
      <div class="section-header">
        <h2>Live Request Stream</h2>
        <div class="stream-controls">
          <button @click="clearRequests" class="clear-btn">🗑️ Clear</button>
          <label class="auto-scroll">
            <input type="checkbox" v-model="autoScroll" />
            Auto-scroll
          </label>
        </div>
      </div>

      <div class="requests-container" ref="requestsContainer">
        <div
          v-for="request in liveRequests"
          :key="request.id"
          class="request-item"
          :class="getRequestStatusClass(request.status_code)"
        >
          <div class="request-header">
            <span class="request-method" :class="getMethodClass(request.method)">
              {{ request.method }}
            </span>
            <span class="request-path">{{ request.path }}</span>
            <span class="request-status" :class="getStatusClass(request.status_code)">
              {{ request.status_code }}
            </span>
            <span class="request-time">{{ request.response_time_ms }}ms</span>
            <span class="request-timestamp">{{ formatTimestamp(request.timestamp) }}</span>
          </div>
          <div class="request-details">
            <div class="detail-row">
              <span class="detail-label">IP:</span>
              <span class="detail-value">{{ request.ip_address || 'Unknown' }}</span>
            </div>
            <div class="detail-row" v-if="request.user_id">
              <span class="detail-label">User:</span>
              <span class="detail-value">{{ request.user_id }}</span>
            </div>
            <div class="detail-row" v-if="request.error_message">
              <span class="detail-label">Error:</span>
              <span class="detail-value error">{{ request.error_message }}</span>
            </div>
          </div>
        </div>

        <div v-if="liveRequests.length === 0" class="no-requests">
          <p>No requests yet. Start monitoring to see live data.</p>
        </div>
      </div>
    </div>

    <!-- System Logs -->
    <div class="logs-section">
      <div class="section-header">
        <h2>System Logs</h2>
        <div class="log-controls">
          <button @click="clearLogs" class="clear-btn">🗑️ Clear</button>
          <select v-model="logLevelFilter" class="log-filter">
            <option value="">All Levels</option>
            <option value="error">Error</option>
            <option value="warn">Warning</option>
            <option value="info">Info</option>
            <option value="debug">Debug</option>
          </select>
        </div>
      </div>

      <div class="logs-container" ref="logsContainer">
        <div
          v-for="log in filteredLogs"
          :key="log.timestamp + log.message"
          class="log-item"
          :class="getLogLevelClass(log.level)"
        >
          <div class="log-header">
            <span class="log-level">{{ log.level.toUpperCase() }}</span>
            <span class="log-target">{{ log.target }}</span>
            <span class="log-timestamp">{{ formatTimestamp(log.timestamp) }}</span>
          </div>
          <div class="log-message">{{ log.message }}</div>
        </div>

        <div v-if="filteredLogs.length === 0" class="no-logs">
          <p>No system logs yet.</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed, nextTick } from 'vue'

interface LiveRequest {
  id: string
  timestamp: string
  method: string
  path: string
  status_code?: number
  response_time_ms?: number
  user_id?: string
  ip_address?: string
  user_agent?: string
  error_message?: string
}

interface SystemLog {
  level: string
  message: string
  timestamp: string
  target: string
}

interface PerformanceMetrics {
  totalRequests: number
  successRate: number
  avgResponseTime: number
  errorRate: number
  activeConnections: number
}

const isConnected = ref(false)
const connecting = ref(false)
const connectionStatus = ref('Disconnected')
const autoScroll = ref(true)
const logLevelFilter = ref('')

const liveRequests = ref<LiveRequest[]>([])
const systemLogs = ref<SystemLog[]>([])
const metrics = reactive<PerformanceMetrics>({
  totalRequests: 0,
  successRate: 0,
  avgResponseTime: 0,
  errorRate: 0,
  activeConnections: 0
})

const requestsContainer = ref<HTMLElement>()
const logsContainer = ref<HTMLElement>()

let websocket: WebSocket | null = null
let reconnectTimer: NodeJS.Timeout | null = null

const filteredLogs = computed(() => {
  if (!logLevelFilter.value) {
    return systemLogs.value
  }
  return systemLogs.value.filter(log => log.level === logLevelFilter.value)
})

const connect = async () => {
  if (connecting.value || isConnected.value) return

  connecting.value = true
  connectionStatus.value = 'Connecting...'

  try {
    const token = localStorage.getItem('adminToken')
    if (!token) {
      throw new Error('No admin token found')
    }

    // Create WebSocket connection
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const wsUrl = `${protocol}//localhost:3000/api/v1/admin/ws`

    websocket = new WebSocket(wsUrl)

    websocket.onopen = () => {
      isConnected.value = true
      connecting.value = false
      connectionStatus.value = 'Connected'
      console.log('WebSocket connected')
    }

    websocket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data)
        handleWebSocketMessage(message)
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    websocket.onclose = () => {
      isConnected.value = false
      connecting.value = false
      connectionStatus.value = 'Disconnected'
      console.log('WebSocket disconnected')

      // Auto-reconnect after 5 seconds
      if (reconnectTimer) clearTimeout(reconnectTimer)
      reconnectTimer = setTimeout(() => {
        if (!isConnected.value) {
          connect()
        }
      }, 5000)
    }

    websocket.onerror = (error) => {
      console.error('WebSocket error:', error)
      connecting.value = false
      connectionStatus.value = 'Connection Error'
    }

  } catch (error) {
    console.error('Failed to connect:', error)
    connecting.value = false
    connectionStatus.value = 'Connection Failed'
  }
}

const disconnect = () => {
  if (websocket) {
    websocket.close()
    websocket = null
  }
  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
}

const handleWebSocketMessage = (message: any) => {
  console.log('Received WebSocket message:', message)
  console.log('Message type:', message.type)

  switch (message.type) {
    case 'AuditLog':
      console.log('Processing AuditLog message:', message)
      handleAuditLog(message)
      break
    case 'SystemLog':
      console.log('Processing SystemLog message:', message)
      handleSystemLog(message)
      break
    case 'PerformanceMetrics':
      console.log('Processing PerformanceMetrics message:', message)
      handlePerformanceMetrics(message)
      break
    case 'ConnectionStatus':
      console.log('Connection status:', message)
      break
    case 'Pong':
      // Handle pong response
      break
    default:
      console.log('Unknown message type:', message)
  }
}

const handleAuditLog = (log: LiveRequest) => {
  console.log('handleAuditLog called with:', log)
  console.log('Current liveRequests count:', liveRequests.value.length)

  liveRequests.value.unshift(log)
  console.log('After adding, liveRequests count:', liveRequests.value.length)

  // Keep only last 100 requests
  if (liveRequests.value.length > 100) {
    liveRequests.value = liveRequests.value.slice(0, 100)
  }

  // Calculate real-time metrics from the audit logs
  calculateMetrics()

  if (autoScroll.value) {
    nextTick(() => {
      if (requestsContainer.value) {
        requestsContainer.value.scrollTop = 0
      }
    })
  }
}

const handleSystemLog = (log: SystemLog) => {
  console.log('handleSystemLog called with:', log)
  systemLogs.value.unshift(log)

  // Keep only last 50 logs
  if (systemLogs.value.length > 50) {
    systemLogs.value = systemLogs.value.slice(0, 50)
  }

  if (autoScroll.value) {
    nextTick(() => {
      if (logsContainer.value) {
        logsContainer.value.scrollTop = 0
      }
    })
  }
}

// Calculate real-time metrics from audit logs
const calculateMetrics = () => {
  if (liveRequests.value.length === 0) {
    // Reset metrics if no requests
    Object.assign(metrics, {
      totalRequests: 0,
      successRate: 0,
      avgResponseTime: 0,
      errorRate: 0,
      activeConnections: 1 // At least 1 for the current connection
    })
    return
  }

  const totalRequests = liveRequests.value.length
  const successfulRequests = liveRequests.value.filter(req =>
    req.status_code && req.status_code >= 200 && req.status_code < 300
  ).length
  const errorRequests = liveRequests.value.filter(req =>
    req.status_code && req.status_code >= 400
  ).length

  const successRate = totalRequests > 0 ? successfulRequests / totalRequests : 0
  const errorRate = totalRequests > 0 ? errorRequests / totalRequests : 0

  const responseTimes = liveRequests.value
    .map(req => req.response_time_ms || 0)
    .filter(time => time > 0)

  const avgResponseTime = responseTimes.length > 0
    ? responseTimes.reduce((sum, time) => sum + time, 0) / responseTimes.length
    : 0

  Object.assign(metrics, {
    totalRequests,
    successRate,
    avgResponseTime,
    errorRate,
    activeConnections: 1 // We'll update this when we implement connection counting
  })
}

const handlePerformanceMetrics = (metricsData: PerformanceMetrics) => {
  Object.assign(metrics, metricsData)
}

const clearRequests = () => {
  liveRequests.value = []
}

const clearLogs = () => {
  systemLogs.value = []
}

const getRequestStatusClass = (statusCode?: number) => {
  if (!statusCode) return ''
  if (statusCode >= 200 && statusCode < 300) return 'status-success'
  if (statusCode >= 400 && statusCode < 500) return 'status-warning'
  if (statusCode >= 500) return 'status-error'
  return ''
}

const getMethodClass = (method: string) => {
  const methodClasses: Record<string, string> = {
    GET: 'method-get',
    POST: 'method-post',
    PUT: 'method-put',
    DELETE: 'method-delete',
    PATCH: 'method-patch'
  }
  return methodClasses[method] || 'method-other'
}

const getStatusClass = (statusCode?: number) => {
  if (!statusCode) return ''
  if (statusCode >= 200 && statusCode < 300) return 'status-success'
  if (statusCode >= 400 && statusCode < 500) return 'status-warning'
  if (statusCode >= 500) return 'status-error'
  return ''
}

const getLogLevelClass = (level: string) => {
  const levelClasses: Record<string, string> = {
    error: 'log-error',
    warn: 'log-warn',
    info: 'log-info',
    debug: 'log-debug'
  }
  return levelClasses[level] || 'log-other'
}

const formatTimestamp = (timestamp: string) => {
  return new Date(timestamp).toLocaleTimeString()
}

onMounted(() => {
  // Auto-connect when component mounts
  connect()
})

onUnmounted(() => {
  disconnect()
})
</script>

<style scoped>
.admin-realtime-monitor {
  padding: 2rem;
}

.monitor-header {
  margin-bottom: 2rem;
}

.monitor-header h1 {
  margin: 0 0 0.5rem 0;
  color: #333;
  font-size: 2rem;
  font-weight: 600;
}

.monitor-header p {
  margin: 0;
  color: #666;
  font-size: 1rem;
}

.connection-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: #f8f9fa;
  padding: 1rem;
  border-radius: 8px;
  margin-bottom: 2rem;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #dc3545;
}

.status-indicator.connected .status-dot {
  background: #28a745;
}

.status-text {
  font-weight: 500;
}

.connection-controls button {
  padding: 0.5rem 1rem;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
}

.connect-btn {
  background: #007bff;
  color: white;
}

.connect-btn:hover:not(:disabled) {
  background: #0056b3;
}

.connect-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.disconnect-btn {
  background: #dc3545;
  color: white;
}

.disconnect-btn:hover {
  background: #c82333;
}

.metrics-section {
  margin-bottom: 2rem;
}

.metrics-section h2 {
  margin: 0 0 1rem 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 1rem;
}

.metric-card {
  display: flex;
  align-items: center;
  gap: 1rem;
  background: white;
  padding: 1.5rem;
  border-radius: 8px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  border: 1px solid #e9ecef;
}

.metric-icon {
  font-size: 2rem;
}

.metric-content h3 {
  margin: 0 0 0.25rem 0;
  color: #666;
  font-size: 0.9rem;
  font-weight: 500;
}

.metric-value {
  margin: 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.requests-section,
.logs-section {
  margin-bottom: 2rem;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 1rem;
}

.section-header h2 {
  margin: 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 600;
}

.stream-controls,
.log-controls {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.clear-btn {
  padding: 0.5rem 1rem;
  background: #6c757d;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9rem;
}

.clear-btn:hover {
  background: #5a6268;
}

.auto-scroll {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.9rem;
  color: #666;
}

.log-filter {
  padding: 0.5rem;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 0.9rem;
}

.requests-container,
.logs-container {
  background: white;
  border: 1px solid #e9ecef;
  border-radius: 8px;
  max-height: 400px;
  overflow-y: auto;
}

.request-item {
  padding: 1rem;
  border-bottom: 1px solid #f1f3f4;
  transition: background-color 0.2s;
}

.request-item:hover {
  background: #f8f9fa;
}

.request-item:last-child {
  border-bottom: none;
}

.request-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}

.request-method {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
}

.method-get { background: #d4edda; color: #155724; }
.method-post { background: #d1ecf1; color: #0c5460; }
.method-put { background: #fff3cd; color: #856404; }
.method-delete { background: #f8d7da; color: #721c24; }
.method-patch { background: #e2e3e5; color: #383d41; }
.method-other { background: #f8f9fa; color: #6c757d; }

.request-path {
  font-family: monospace;
  font-size: 0.9rem;
  color: #333;
  flex: 1;
}

.request-status {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

.status-success { background: #d4edda; color: #155724; }
.status-warning { background: #fff3cd; color: #856404; }
.status-error { background: #f8d7da; color: #721c24; }

.request-time {
  font-size: 0.8rem;
  color: #666;
}

.request-timestamp {
  font-size: 0.8rem;
  color: #999;
}

.request-details {
  font-size: 0.85rem;
  color: #666;
}

.detail-row {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.25rem;
}

.detail-label {
  font-weight: 500;
  min-width: 60px;
}

.detail-value {
  font-family: monospace;
}

.detail-value.error {
  color: #dc3545;
}

.log-item {
  padding: 1rem;
  border-bottom: 1px solid #f1f3f4;
}

.log-item:last-child {
  border-bottom: none;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 0.5rem;
  flex-wrap: wrap;
}

.log-level {
  padding: 0.25rem 0.5rem;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
}

.log-error { background: #f8d7da; color: #721c24; }
.log-warn { background: #fff3cd; color: #856404; }
.log-info { background: #d1ecf1; color: #0c5460; }
.log-debug { background: #e2e3e5; color: #383d41; }
.log-other { background: #f8f9fa; color: #6c757d; }

.log-target {
  font-family: monospace;
  font-size: 0.8rem;
  color: #666;
}

.log-timestamp {
  font-size: 0.8rem;
  color: #999;
}

.log-message {
  font-size: 0.9rem;
  color: #333;
  line-height: 1.4;
}

.no-requests,
.no-logs {
  padding: 2rem;
  text-align: center;
  color: #666;
}

@media (max-width: 768px) {
  .admin-realtime-monitor {
    padding: 1rem;
  }

  .metrics-grid {
    grid-template-columns: 1fr;
  }

  .request-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  .log-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }
}
</style>