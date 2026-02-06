<template>
  <nav>
    <ul class="nav-list">
      <li class="dropdown">
        <a href="#">Menu</a>
        <div class="dropdown-content">
          <a href="#">Home</a>
          <a href="#">Assignments</a>
          <a href="#">Gambling Mode</a>
        </div>
      </li>
    </ul>

    <div class="profile">
      <img src="https://via.placeholder.com/40x40/666/fff?text=US" alt="User Avatar" style="border-radius: 50%;">
      <span>Student Name</span>
      <div class="dropdown">
        <div class="dropdown-content">
          <a href="#">Profile</a>
          <a href="#">Settings</a>
          <button @click="handleLogout" class="logout-btn">Logout</button>
        </div>
      </div>
    </div>
  </nav>
</template>

<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()
const loading = ref(false)

const handleLogout = async () => {
  loading.value = true
  try {
    const response = await fetch('/auth/logout', {
      method: 'POST',
      credentials: 'include',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({})
    })
    if (response.ok) {
      localStorage.removeItem('user')  
      router.push('/login')  
    }
  } catch (error) {
    console.error('Logout failed:', error)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.logout-btn {
  background: none;
  border: none;
  width: 100%;
  padding: 12px 16px;
  text-align: left;
  cursor: pointer;
  font-size: inherit;
  color: inherit;
}
.logout-btn:hover {
  background-color: #f1f1f1;
}
.logout-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
