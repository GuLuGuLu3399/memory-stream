// 用途：应用路由配置，定义编辑器页面和重定向规则
import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    redirect: '/editor',
  },
  {
    path: '/editor/:uuid?',
    name: 'editor',
    component: () => import('@/views/EditorPage.vue'),
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/editor',
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
