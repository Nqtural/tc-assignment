import { createRouter, createWebHistory } from "vue-router";
//import Home from '../components/Home.vue'
import login from '../components/login.vue'

const Login = () => import("../components/login.vue");

const routes = [
	//{ path: "/home", component: Home },
	{ path: "/login", component: Login },
]

export default createRouter({
  history: createWebHistory(),
  routes
});
