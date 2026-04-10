//! ast-gen: 从 `RawMd` 生成真实 AST JSON，直接写入 SQL 文件
//!
//! 用法: cargo run -p ast-gen [输出路径]
//! 默认写入 ../../go-server/migration/sql/009_seed_ast_data.sql

use std::fmt::Write;

struct SeedCard {
    id: &'static str,
    raw_md: &'static str,
}

// CC-理由: 主函数包含多个种子卡片定义，拆分会降低可读性
#[allow(clippy::too_many_lines)]
fn main() {
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../go-server/migration/sql/009_seed_ast_data.sql".to_string());

    let cards = vec![
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000001",
            raw_md: r"# 我的知识系统规划

## 核心目标
- 建立可复用的知识图谱
- 实现语义缩放和双线视觉

## 技术栈
- 后端: Go + PostgreSQL
- 前端: Vue 3 + Vue Flow",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000002",
            raw_md: r"# Phase 1: 基础架构

## 完成项
- [x] 数据库设计
- [x] Go 后端 API
- [x] Vue Flow 集成

## 核心收获
- 理解了 UUID 与前端 ID 的映射
- 掌握了 GORM 的软删除机制",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000003",
            raw_md: r"# Phase 2: 语义缩放

## 目标
- 实现 0.5x (Topic) 视图
- 实现 1.0x (Cluster) 视图
- 实现 2.0x (Detail) 视图

## 关键技术
- Vue Flow 的 parentNode 机制
- 前端动态样式计算",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000004",
            raw_md: r"# Vue Flow 最佳实践

## 官方文档
- [Vue Flow 官网](https://vueflow.dev/)
- [示例库](https://github.com/bcakmakoglu/vue-flow-examples)

## 核心概念
- Node Types
- Edge Types
- Handle 系统",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000005",
            raw_md: r"# Go 后端性能优化

## 已实施
- 热度计算: O(1) 单表操作
- UUID 解析: resolveIdentifier 复用
- 布局缓存: card_layouts 表

## 待优化
- 使用 Redis 缓存热点数据
- 实现增量布局计算",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000006",
            raw_md: r"# PostgreSQL UUID 性能

## 关键发现
- UUID 类型比 VARCHAR(36) 快 15%
- gen_random_uuid() 比客户端生成更快
- B-Tree 索引在 UUID 上非常高效

## 推荐配置
```sql
CREATE INDEX idx_cards_id ON cards(id);
```",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000007",
            raw_md: r"# Phase 3: 联调与优化

## 当前状态
- [x] 修复 UUID 类型冲突
- [x] 优化热度计算 SQL
- [x] 添加 root 别名支持

## 下一步
- [ ] 前端联调测试
- [ ] 性能压测
- [ ] 用户体验优化",
        },
        SeedCard {
            id: "a0000000-0000-0000-0000-000000000008",
            raw_md: r"# Rust 学习笔记

## 所有权规则
1. 每个值都有一个所有者
2. 同一时刻只能有一个所有者
3. 当所有者离开作用域，值被丢弃

## 借用规则
- 不可变借用可以有多个
- 可变借用只能有一个",
        },
    ];

    let mut sql = String::new();
    sql.push_str("-- 🌟 009: 用真实 AST 数据替换占位符\n");
    sql.push_str("-- 由 ast-gen 自动生成，请勿手动编辑\n");
    sql.push_str("BEGIN;\n\n");

    for card in &cards {
        match md_parser::parse_markdown(card.raw_md) {
            Ok(ast) => {
                let ast_json = serde_json::to_string(&ast).unwrap();
                let escaped = ast_json.replace('\'', "''");
                writeln!(
                    sql,
                    "UPDATE cards SET ast_data = '{}'::JSONB WHERE id = '{}'::UUID;",
                    escaped, card.id
                ).unwrap();
            }
            Err(e) => {
                eprintln!("-- ❌ 解析失败 [{}]: {}", card.id, e);
            }
        }
    }

    sql.push_str("\nCOMMIT;\n");

    std::fs::write(&out_path, &sql).expect("写入文件失败");
    eprintln!("✅ 已写入: {out_path}");
}
