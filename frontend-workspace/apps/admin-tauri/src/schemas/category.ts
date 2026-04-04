import { z } from 'zod'

/** Single Category schema matching Go backend Category struct */
export const CategorySchema = z.object({
  id: z.number(),
  name: z.string(),
  description: z.string().optional().default(''),
  parent_id: z.number().nullable(),
  sort_order: z.number().optional().default(0),
  created_at: z.string(),
  updated_at: z.string().optional().default(''),
  theme_color: z.string().nullable().optional(),
})

/** Category list response from GET /categories */
export const CategoryListResponseSchema = z.object({
  categories: z.array(CategorySchema),
})

/** Category tree node (recursive) for GET /categories/tree */
export const CategoryTreeNodeSchema: z.ZodType<CategoryTreeNodeData> = z.lazy(() =>
  z.object({
    id: z.number(),
    name: z.string(),
    description: z.string().optional().default(""),
    parent_id: z.number().nullable(),
    sort_order: z.number().optional().default(0),
    created_at: z.string(),
    children: z.array(CategoryTreeNodeSchema),
  })
)

/** Type for recursive tree node — used in CategoryTreeNodeSchema and CategoryTreeNode.vue */
export interface CategoryTreeNodeData {
  id: number
  name: string
  description?: string
  parent_id: number | null
  sort_order?: number
  created_at: string
  children: CategoryTreeNodeData[]
}

export const CategoryTreeResponseSchema = z.object({
  categories: z.array(CategoryTreeNodeSchema),
})

/** Create category request */
export const CreateCategoryRequestSchema = z.object({
  name: z.string().min(1, '分类名称不能为空').max(100),
  description: z.string().optional(),
  parent_id: z.number().nullable().optional(),
})

/** Update category request */
export const UpdateCategoryRequestSchema = z.object({
  name: z.string().min(1, '分类名称不能为空').max(100),
  description: z.string().optional(),
  parent_id: z.number().nullable().optional(),
})
