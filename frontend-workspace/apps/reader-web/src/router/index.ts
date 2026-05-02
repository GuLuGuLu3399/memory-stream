// ────────────────────────────────────────────────────────────────
// router/index.ts — Reader web routes
// ────────────────────────────────────────────────────────────────

import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'welcome',
      component: () => import('@/pages/WelcomePage.vue'),
    },
    {
      path: '/search',
      name: 'search',
      component: () => import('@/pages/SearchPage.vue'),
    },
    {
      path: '/card/:uuid',
      name: 'card',
      component: () => import('@/pages/ReaderPage.vue'),
    },
    {
      path: '/graph',
      name: 'graph',
      component: () => import('@/pages/GlobalGraphPage.vue'),
    },
    {
      path: '/category/:category',
      name: 'category',
      component: () => import('@/pages/CategoryPage.vue'),
    },
  ],
  scrollBehavior() {
    return { top: 0 }
  },
})

export default router
