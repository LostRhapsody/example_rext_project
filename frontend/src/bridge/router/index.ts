import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '@/views/HomeView.vue'
import LoginView from '@/views/LoginView.vue'
import RegisterView from '@/views/RegisterView.vue'
import ProfileView from '@/views/ProfileView.vue'
import AdminLoginView from '@/views/AdminLoginView.vue'
import AdminLayout from '@/components/AdminLayout.vue'
import AdminDashboardView from '@/views/AdminDashboardView.vue'
import AdminLogsView from '@/views/AdminLogsView.vue'
import AdminUsersView from '@/views/AdminUsersView.vue'
import AdminDatabaseView from '@/views/AdminDatabaseView.vue'
import AdminHealthView from '@/views/AdminHealthView.vue'
import AdminRealtimeMonitorView from '@/views/AdminRealtimeMonitorView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView,
    },
    {
      path: '/login',
      name: 'login',
      component: LoginView,
    },
    {
      path: '/register',
      name: 'register',
      component: RegisterView,
    },
    {
      path: '/profile',
      name: 'profile',
      component: ProfileView,
      meta: { requiresAuth: true },
    },
    // Admin routes
    {
      path: '/admin/login',
      name: 'admin-login',
      component: AdminLoginView,
    },
    {
      path: '/admin',
      component: AdminLayout,
      meta: { requiresAdmin: true },
      children: [
        {
          path: '',
          name: 'admin-dashboard',
          component: AdminDashboardView,
        },
        {
          path: 'logs',
          name: 'admin-logs',
          component: AdminLogsView,
        },
        {
          path: 'users',
          name: 'admin-users',
          component: AdminUsersView,
        },
        {
          path: 'database',
          name: 'admin-database',
          component: AdminDatabaseView,
        },
        {
          path: 'health',
          name: 'admin-health',
          component: AdminHealthView,
        },
        {
          path: 'monitor',
          name: 'admin-realtime-monitor',
          component: AdminRealtimeMonitorView,
        },
      ],
    },
  ],
})

// Navigation guard for protected routes
router.beforeEach((to, from, next) => {
  const token = localStorage.getItem('token')
  const adminToken = localStorage.getItem('adminToken')

  // Check for admin routes
  if (to.meta.requiresAdmin) {
    if (!adminToken) {
      next({ name: 'admin-login' })
      return
    }
  }

  // Check for regular auth routes
  if (to.meta.requiresAuth && !token) {
    next({ name: 'login' })
    return
  }

  next()
})

export default router
