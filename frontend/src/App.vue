<script setup>
import Login from './components/login.vue'
import Navbar from './components/Navbar.vue';
import Home from './components/Home.vue';
</script>

<template>
  <router-link to="/login">Login</router-link> |
  <router-view />
</template>

<style scoped></style>
