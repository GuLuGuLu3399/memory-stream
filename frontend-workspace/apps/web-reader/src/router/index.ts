/**
 * Router — 血肉神殿路径指引
 *
 * /       → redirect → /list
 * /list   → ListView（血脊列表）
 * /graph  → GraphView（图谱视野）
 *
 * ?card=<id> query param 同步 DetailDrawer 选中状态
 */

import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/",
      redirect: "/graph",
    },
    {
      path: "/list",
      name: "list",
      component: () => import("../views/ListView.vue"),
    },
    {
      path: "/graph",
      name: "graph",
      component: () => import("../views/GraphView.vue"),
    },
  ],
});

export default router;
