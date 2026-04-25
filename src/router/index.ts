import { createRouter, createWebHashHistory } from 'vue-router'
import HeroDeployPage from '../pages/HeroDeployPage.vue'

export const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      name: 'hero-deploy',
      component: HeroDeployPage,
    },
  ],
})
