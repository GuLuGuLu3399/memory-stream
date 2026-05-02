// ────────────────────────────────────────────────────────────────
// Tree utilities — directory/category tree construction and search
// 树形工具 — 目录/分类树的构建和搜索
// ────────────────────────────────────────────────────────────────

export interface TreeNode {
  id: string;
  name: string;
  parent_id: string | null;
  is_dir: boolean;
  children: TreeNode[];
}

export interface FlatTreeItem {
  id: string;
  name: string;
  parent_id: string | null;
  is_dir: boolean;
}

export function buildTree(items: FlatTreeItem[]): TreeNode[] {
  const map = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];

  for (const item of items) {
    map.set(item.id, { ...item, children: [] });
  }

  for (const item of items) {
    const node = map.get(item.id)!;
    if (item.parent_id && map.has(item.parent_id)) {
      map.get(item.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}

export function findInTree(
  tree: TreeNode[],
  predicate: (node: TreeNode) => boolean,
): TreeNode | null {
  for (const node of tree) {
    if (predicate(node)) return node;
    const found = findInTree(node.children, predicate);
    if (found) return found;
  }
  return null;
}

export function flattenTree(tree: TreeNode[]): FlatTreeItem[] {
  const result: FlatTreeItem[] = [];
  function walk(nodes: TreeNode[]) {
    for (const node of nodes) {
      result.push({ id: node.id, name: node.name, parent_id: node.parent_id, is_dir: node.is_dir });
      walk(node.children);
    }
  }
  walk(tree);
  return result;
}

export function countNodes(tree: TreeNode[]): number {
  let count = 0;
  function walk(nodes: TreeNode[]) {
    for (const node of nodes) {
      count++;
      walk(node.children);
    }
  }
  walk(tree);
  return count;
}
