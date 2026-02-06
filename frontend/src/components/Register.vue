<template>
    <div class="register-container">
        <h2>Register</h2>

        <form @submit.prevent="handleRegister">
            <div class="field">
                <label>Instans </label>
                <input
                type="instans"
                v-model="instans"
                required 
                placeholder="Instans"
            </div>

            <div class="field">
                <label>Förnamn </label>
                <input
                type="förnamn"
                v-model="förnamn"
                required
                placeholder="Förnamn"
                />
            </div>

            <div class="field">
                <label>Efternamn </label>
                <input
                type="efternamn"
                v-model="efternamn"
                required
                placeholder="Efternamn"
                />
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

        <button type="submit">Register</button>
        </form>

        <p v-if="error" class="error">{{ error }}</p>
    </div>
</template>

<script setup>
import { ref } from "vue";

const instans = ref("");
const förnamn = ref("");
const efternamn = ref("");
const email = ref("");
const password = ref("");
const error = ref("");

    async function handleRegister() {
        error.value = "";

        if (!instans.value || !förnamn.value || !efternamn.value || !email.value || !password.value) {
            error.value = "Please fill in all fields";
            return;
        }

        try {
            const res = await fetch(`http://${instans.value}/auth/register`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                name: förnamn.value,
                surname: efternamn.value,
                email: email.value,
                password: password.value,
            }),
        });

        const data = await res.json();

        console.log(data);

        if (res.ok) {
            console.log("Register success:", data);
            window.location.href = "/";
        } else {
            error.value = data.message || "Registration failed";
        }
    } catch (err) {
            error.value = "An error occurred. Please try again.";
            console.error("Registration error:", err);

	}
}
</script>

<style scoped>
</style>
