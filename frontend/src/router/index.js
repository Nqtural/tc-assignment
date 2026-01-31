import { createRouter, createWebHistory } from "vue-router";
import Home from "../components/Home.vue";
import Login from "../components/Login.vue";
import Register from "../components/Register.vue";

const routes = [
	{ path: "/home", component: Home },
	{ path: "/login", component: Login },
	{ path: "/register", component: Register },
];

export default createRouter({
	history: createWebHistory(),
	routes,
});
