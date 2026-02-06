<template>
  <div class="login-container">
    <h2>Login</h2>

    <form @submit.prevent="handleLogin">
      <div class="field">
        <label>Instans </label>
        <input
          type="instans"
          v-model="instans"
          required 
          placeholder="Instans"
      </div>

      <div class="field">
        <label>Email </label>
        <input
          type="email"
          v-model="email"
          required
          placeholder="you@example.com"
        />
      </div>

      <div class="field">
        <label>Password </label>
        <input
          type="password"
          v-model="password"
          required
          placeholder="••••••••"
        />
      </div>

      <button type="submit">Login</button>
    </form>

    <p v-if="error" class="error">{{ error }}</p>
  </div>
</template>

<script setup>
import { ref } from "vue";

const instans = ref("");
const email = ref("");
const password = ref("");
const error = ref("");

function handleLogin() {
	error.value = "";

	// basic example validation
	if (!instans.value || !email.value || !password.value) {
		error.value = "Please fill in all fields";
		return;
	}

  const payload = {
    email: email.value,
    password: password.value,
  };

  console.log(payload);

  fetch(`http://${instans.value}/auth/login`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  })
    .then(res => {
      if (!res.ok) {
        // status-based handling
        if (res.status === 404) {
          console.log("Login failed: user does not exist");
          error.value = "User does not exist";
        } else if (res.status === 401 || res.status === 403) {
          console.log("Login failed: invalid credentials");
          error.value = "Invalid credentials";
        } else if (res.status >= 500) {
          console.log("Login failed: internal server error");
          error.value = "Internal server error";
        } else {
          console.log("Login failed: unknown error", res.status);
          error.value = "Login failed";
        }
        return;
      }

      console.log("Login request accepted");
      return res.json();
    })
    .then(data => {
      if (!data) return;

      console.log("Login success:", data);
      window.location.href = "/";
      
    })
    .catch(() => {
      console.log("Login failed: network error");
      error.value = "Network error";
    });

}
</script>

<style scoped>
</style>
