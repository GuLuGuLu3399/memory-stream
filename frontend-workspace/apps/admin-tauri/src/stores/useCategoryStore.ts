/**
 * @module useCategoryStore
 *
 * 分类管理 Store — Category CRUD 操作
 *
 * 所有网络请求通过 Rust 的 `api_request` IPC 命令发出。
 */

import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "./useToast";
import { CategoryListResponseSchema } from "../schemas/category";
import { z } from "zod";
import type { Category } from "@memory-stream/types";

export const useCategoryStore = defineStore("category", () => {
  // ---- 响应式状态 ----
  const categories = ref<Category[]>([]);

  // ---- Category 操作 — 全部通过 Rust api_request 网关 ----

  /** 从后端加载全部分类列表 */
  async function loadCategories() {
    try {
      const data = await invoke<Record<string, unknown>>("api_request", {
        method: "GET",
        endpoint: "/categories",
      });
      // Validate with Zod
      const parsed = CategoryListResponseSchema.parse(data);
      categories.value = parsed.categories.map((c) => ({
        id: c.id as number,
        name: c.name || "",
        description: c.description || "",
        parent_id: c.parent_id ?? null,
        created_at: c.created_at || "",
        updated_at: c.updated_at ?? "",
        theme_color: c.theme_color ?? null,
      }));
    } catch (e) {
      if (e instanceof z.ZodError) {
        // Validation failed – log and toast without blocking
        console.error("[CategoryStore] Zod validation failed:", e.errors);
        useToast().addToast("分类数据格式异常", "error");
      } else {
        console.error("[CategoryStore] loadCategories failed:", e);
      }
    }
  }

  /** 创建新分类 */
  async function createCategory(name: string, description: string = "") {
    const toast = useToast();
    try {
      await invoke("api_request", {
        method: "POST",
        endpoint: "/categories",
        body: { name, description },
      });
      toast.addToast("分类已创建 ✓", "success");
      await loadCategories();
    } catch (e) {
      toast.addToast("创建分类失败: " + String(e), "error");
    }
  }

  /** 更新分类名称和描述 */
  async function updateCategory(
    id: number,
    name: string,
    description: string = "",
    theme_color: string | null = null,
  ) {
    const toast = useToast();
    try {
      await invoke("api_request", {
        method: "PUT",
        endpoint: `/categories/${id}`,
        body: { name, description, theme_color },
      });
      toast.addToast("分类已更新 ✓", "success");
      await loadCategories();
    } catch (e) {
      toast.addToast("更新分类失败: " + String(e), "error");
    }
  }

  /** 删除指定分类 */
  async function deleteCategory(id: number) {
    const toast = useToast();
    try {
      await invoke("api_request", {
        method: "DELETE",
        endpoint: `/categories/${id}`,
      });
      toast.addToast("分类已删除 ✓", "success");
      await loadCategories();
    } catch (e) {
      toast.addToast("删除分类失败: " + String(e), "error");
    }
  }

  return {
    categories,
    loadCategories,
    createCategory,
    updateCategory,
    deleteCategory,
  };
});
