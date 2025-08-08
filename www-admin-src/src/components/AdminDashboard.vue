<script setup>
import { ref, reactive, onMounted } from 'vue'
import ConfigEditor from './ConfigEditor.vue'

// Define props and emits
const props = defineProps({
  user: {
    type: Object,
    required: true
  }
})

const emit = defineEmits(['logout'])

// Dashboard state
const isLoading = ref(false)
const error = ref('')
const activeTab = ref('dashboard')

// Server stats
const stats = reactive({
  uptime: '0 minutes',
  requests: 0,
  activeConnections: 0,
  lastUpdated: new Date()
})

// Load server configuration (for stats only)
const loadServerStats = async () => {
  isLoading.value = true
  error.value = ''

  try {
    const response = await fetch('/config', {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${props.user.sessionToken}`,
        'Content-Type': 'application/json'
      }
    })

    if (response.ok) {
      // We can use this for stats if needed in the future
      // For now just clear any errors
      error.value = ''
    } else {
      error.value = 'Failed to load server stats'
    }
  } catch (err) {
    console.error('Server stats loading error:', err)
    error.value = 'Network error while loading server stats'
  } finally {
    isLoading.value = false
  }
}

// Mock function to update stats (in a real app, this would fetch from the server)
const updateStats = () => {
  stats.lastUpdated = new Date()
  // These would be real metrics in a production environment
}

// Handle logout
const handleLogout = () => {
  emit('logout')
}

// Tab navigation
const setActiveTab = (tab) => {
  activeTab.value = tab
}

// Close configuration editor
const closeConfigEditor = () => {
  showConfigEditor.value = false
  // Reload config after potential changes
  loadConfiguration()
}

// Initialize dashboard
onMounted(() => {
  updateStats()
  setInterval(updateStats, 30000) // Update stats every 30 seconds
})
</script>

<template>
  <div class="dashboard">
    <!-- Header -->
    <header class="dashboard-header">
      <div class="header-content">
        <div class="header-left">
          <h1>Grux Administration</h1>
        </div>
        <div class="header-right">
          <button @click="handleLogout" class="logout-button">
            <span class="logout-icon">🚪</span>
            Logout
          </button>
        </div>
      </div>
    </header>

    <!-- Navigation -->
    <nav class="dashboard-nav">
      <button
        :class="['nav-button', { active: activeTab === 'dashboard' }]"
        @click="setActiveTab('dashboard')"
      >
        📊 Dashboard
      </button>
      <button
        :class="['nav-button', { active: activeTab === 'config' }]"
        @click="setActiveTab('config')"
      >
        ⚙️ Configuration
      </button>
      <button
        :class="['nav-button', { active: activeTab === 'logs' }]"
        @click="setActiveTab('logs')"
      >
        📝 Logs
      </button>
    </nav>

    <!-- Main Content -->
    <main class="dashboard-main">
      <!-- Dashboard Tab -->
      <div v-if="activeTab === 'dashboard'" class="dashboard-content">
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-icon">🚀</div>
            <div class="stat-content">
              <h3>Server Status</h3>
              <p class="stat-value">Running</p>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">⏱️</div>
            <div class="stat-content">
              <h3>Uptime</h3>
              <p class="stat-value">{{ stats.uptime }}</p>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">📈</div>
            <div class="stat-content">
              <h3>Requests Today</h3>
              <p class="stat-value">{{ stats.requests.toLocaleString() }}</p>
            </div>
          </div>

          <div class="stat-card">
            <div class="stat-icon">🔗</div>
            <div class="stat-content">
              <h3>Active Connections</h3>
              <p class="stat-value">{{ stats.activeConnections }}</p>
            </div>
          </div>
        </div>

        <div class="info-section">
          <h2>System Information</h2>
          <div class="info-grid">
            <div class="info-item">
              <strong>Server Version:</strong> Grux v0.1.0
            </div>
            <div class="info-item">
              <strong>Admin Portal:</strong> Enabled
            </div>
            <div class="info-item">
              <strong>Last Updated:</strong> {{ stats.lastUpdated.toLocaleString() }}
            </div>
          </div>
        </div>
      </div>

      <!-- Configuration Tab -->
      <div v-else-if="activeTab === 'config'" class="config-content">
        <!-- Inline Configuration Editor -->
        <div class="inline-config-editor">
          <ConfigEditor :user="user" :inline="true" />
        </div>
      </div>

      <!-- Logs Tab -->
      <div v-else-if="activeTab === 'logs'" class="logs-content">
        <h2>Server Logs</h2>
        <div class="feature-placeholder">
          <div class="placeholder-icon">📝</div>
          <h3>Log Viewer</h3>
          <p>Log viewing functionality will be implemented in a future version.</p>
          <p>For now, please check the server logs directly on the file system.</p>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.dashboard {
  min-height: 100vh;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  display: flex;
  flex-direction: column;
}

.dashboard-header {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  padding: 1rem 2rem;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 1200px;
  margin: 0 auto;
}

.header-left h1 {
  margin: 0;
  color: #333;
  font-size: 1.8rem;
  font-weight: 700;
}

.header-left p {
  margin: 0.25rem 0 0 0;
  color: #666;
  font-size: 1rem;
}

.logout-button {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, #ff6b6b, #ee5a24);
  color: white;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.logout-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(255, 107, 107, 0.3);
}

.logout-icon {
  font-size: 1.2rem;
}

.dashboard-nav {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  padding: 1rem 2rem;
  border-bottom: 1px solid rgba(0, 0, 0, 0.1);
  display: flex;
  gap: 1rem;
  justify-content: center;
}

.nav-button {
  padding: 0.75rem 1.5rem;
  background: transparent;
  border: 2px solid transparent;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  color: #666;
}

.nav-button:hover {
  color: #333;
  background: rgba(102, 126, 234, 0.1);
}

.nav-button.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-color: transparent;
}

.dashboard-main {
  flex: 1;
  padding: 2rem;
  max-width: 1200px;
  margin: 0 auto;
  width: 100%;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 1.5rem;
  margin-bottom: 2rem;
}

.stat-card {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 15px;
  padding: 1.5rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.stat-icon {
  font-size: 2.5rem;
  flex-shrink: 0;
}

.stat-content h3 {
  margin: 0 0 0.5rem 0;
  color: #666;
  font-size: 0.9rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-value {
  margin: 0;
  color: #333;
  font-size: 1.5rem;
  font-weight: 700;
}

.info-section {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 15px;
  padding: 2rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.info-section h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.4rem;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.info-item {
  padding: 1rem;
  background: rgba(102, 126, 234, 0.05);
  border-radius: 10px;
  border: 1px solid rgba(102, 126, 234, 0.1);
}

.config-content,
.logs-content {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(10px);
  border-radius: 15px;
  padding: 2rem;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.config-content h2,
.logs-content h2 {
  margin: 0 0 1.5rem 0;
  color: #333;
  font-size: 1.4rem;
}

.loading-message {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 2rem;
  text-align: center;
  justify-content: center;
}

.loading-spinner {
  width: 20px;
  height: 20px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #667eea;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.error-message {
  background: #fee2e2;
  border: 1px solid #fecaca;
  color: #dc2626;
  padding: 1rem;
  border-radius: 10px;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 1rem;
}

.retry-button,
.load-config-button {
  padding: 0.5rem 1rem;
  background: #667eea;
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
}

.config-actions {
  display: flex;
  gap: 1rem;
  margin-bottom: 1.5rem;
}

.edit-config-button {
  padding: 0.75rem 1.5rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.edit-config-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(102, 126, 234, 0.3);
}

.refresh-button {
  padding: 0.75rem 1.5rem;
  background: #10b981;
  color: white;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.refresh-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 16px rgba(16, 185, 129, 0.3);
}

.config-viewer {
  background: rgba(255, 255, 255, 0.7);
  border-radius: 10px;
  padding: 1.5rem;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.config-viewer h3 {
  margin: 0 0 1rem 0;
  color: #333;
}

.config-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}

.config-modal-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
}

.config-modal-content {
  position: relative;
  width: 100%;
  max-width: 1200px;
  max-height: 90vh;
  z-index: 1001;
}

.config-note {
  background: rgba(255, 193, 7, 0.1);
  border: 1px solid rgba(255, 193, 7, 0.3);
  color: #856404;
  padding: 1rem;
  border-radius: 10px;
  margin-bottom: 1rem;
}

.config-text {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 10px;
  padding: 1rem;
  white-space: pre-wrap;
  overflow-x: auto;
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
  line-height: 1.4;
}

.feature-placeholder {
  text-align: center;
  padding: 3rem;
  color: #666;
}

.placeholder-icon {
  font-size: 4rem;
  margin-bottom: 1rem;
}

.feature-placeholder h3 {
  margin: 0 0 1rem 0;
  color: #333;
}

.feature-placeholder p {
  margin: 0.5rem 0;
  max-width: 500px;
  margin-left: auto;
  margin-right: auto;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* Responsive design */
@media (max-width: 768px) {
  .dashboard-nav {
    flex-wrap: wrap;
    justify-content: center;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .header-content {
    flex-direction: column;
    gap: 1rem;
    text-align: center;
  }

  .dashboard-main {
    padding: 1rem;
  }
}

.config-content {
  background: #f8fafc;
  min-height: calc(100vh - 200px);
  padding: 1.5rem;
}
</style>
