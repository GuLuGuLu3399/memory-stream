<script setup lang="ts">
// 用途：应用根组件，渲染应用布局和路由视图
import AppLayout from '@/components/system/AppLayout.vue'
</script>

<template>
  <AppLayout>
    <RouterView />
  </AppLayout>
</template>
