<template>
  <div class="admin-layout">
    <!-- Admin Header -->
    <header class="admin-header">
      <div class="header-content">
        <div class="logo">
          <h1>Rext Admin Panel</h1>
        </div>
        <div class="admin-info">
          <span class="admin-email">{{ adminEmail }}</span>
          <button @click="logout" class="logout-btn">Logout</button>
        </div>
      </div>
    </header>

    <!-- Admin Navigation -->
    <nav class="admin-nav">
      <div class="nav-content">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: $route.path === item.path }"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span class="nav-text">{{ item.label }}</span>
        </router-link>
      </div>
    </nav>

    <!-- Main Content Area -->
    <main class="admin-main">
      <div class="main-content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { adminLogoutHandler } from '@/bridge/client'

const router = useRouter()
const adminEmail = ref(localStorage.getItem('adminEmail') || 'Admin')

// Navigation items
const navItems = ref([
  {
    path: '/admin',
    label: 'Dashboard',
    icon: '📊'
  },
  {
    path: '/admin/logs',
    label: 'Request Logs',
    icon: '📋'
  },
  {
    path: '/admin/users',
    label: 'User Management',
    icon: '👥'
  },
  {
    path: '/admin/roles',
    label: 'Role Management',
    icon: '🔑'
  },
  {
    path: '/admin/database',
    label: 'Database',
    icon: '🗄️'
  },
  {
    path: '/admin/health',
    label: 'System Health',
    icon: '💚'
  },
  {
    path: '/admin/monitor',
    label: 'Real-time Monitor',
    icon: '📡'
  }
])

const logout = async () => {
  try {
    const token = localStorage.getItem('adminToken')
    if (!token) {
      router.push('/admin/login')
      return
    }

    await adminLogoutHandler({
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    localStorage.removeItem('adminToken')
    localStorage.removeItem('adminEmail')
    router.push('/admin/login')
  } catch (error) {
    console.error('Logout error:', error)
    // Even if logout fails, clear local storage and redirect
    localStorage.removeItem('adminToken')
    localStorage.removeItem('adminEmail')
    router.push('/admin/login')
  }
}
</script>

<style scoped>
.admin-layout {
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5;
}

.admin-header {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  padding: 1rem 2rem;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
}

.logo h1 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}

.admin-info {
  display: flex;
  align-items: center;
  gap: 1rem;
}

.admin-email {
  font-size: 0.9rem;
  opacity: 0.9;
}

.logout-btn {
  background: rgba(255,255,255,0.2);
  border: 1px solid rgba(255,255,255,0.3);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.logout-btn:hover {
  background: rgba(255,255,255,0.3);
}

.admin-nav {
  background: white;
  border-bottom: 1px solid #e0e0e0;
  padding: 0 2rem;
}

.nav-content {
  display: flex;
  max-width: 1200px;
  margin: 0 auto;
  gap: 0.5rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 1rem 1.5rem;
  text-decoration: none;
  color: #666;
  border-bottom: 3px solid transparent;
  transition: all 0.2s;
  font-weight: 500;
}

.nav-item:hover {
  color: #667eea;
  background-color: #f8f9ff;
}

.nav-item.active {
  color: #667eea;
  border-bottom-color: #667eea;
  background-color: #f8f9ff;
}

.nav-icon {
  font-size: 1.2rem;
}

.nav-text {
  font-size: 0.9rem;
}

.admin-main {
  flex: 1;
  overflow: auto;
  padding: 2rem;
}

.main-content {
  max-width: 1200px;
  margin: 0 auto;
  background: white;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  min-height: calc(100vh - 200px);
}

@media (max-width: 768px) {
  .header-content {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .nav-content {
    flex-wrap: wrap;
    justify-content: center;
  }

  .nav-item {
    padding: 0.75rem 1rem;
  }

  .admin-main {
    padding: 1rem;
  }
}
</style>