BEGIN;
INSERT INTO cards (
        id,
        title,
        raw_md,
        ast_data,
        ast_version,
        category_name
    )
VALUES (
        'a0000000-0000-0000-0000-000000000001'::UUID,
        '我的知识系统规划',
        '# 我的知识系统规划',
        '{"type":"Root"}'::JSONB,
        '1.0.0',
        NULL
    ),
    (
        'a0000000-0000-0000-0000-000000000002'::UUID,
        'Phase 1 - 基础架构',
        '# Phase 1',
        '{"type":"Root"}'::JSONB,
        '1.0.0',
        '架构'
    ),
    (
        'a0000000-0000-0000-0000-000000000003'::UUID,
        'Phase 2 - 语义缩放',
        '# Phase 2',
        '{"type":"Root"}'::JSONB,
        '1.0.0',
        '架构'
    ),
    (
        'a0000000-0000-0000-0000-000000000004'::UUID,
        'Vue Flow 最佳实践',
        '# Vue Flow',
        '{"type":"Root"}'::JSONB,
        '1.0.0',
        '工具链'
    ),
    (
        'a0000000-0000-0000-0000-000000000005'::UUID,
        'Rust 学习笔记',
        '# Rust',
        '{"type":"Root"}'::JSONB,
        '1.0.0',
        'Rust'
    ) ON CONFLICT (id) DO NOTHING;
INSERT INTO card_edges (source_id, target_id, relation_type)
VALUES (
        'a0000000-0000-0000-0000-000000000001'::UUID,
        'a0000000-0000-0000-0000-000000000002'::UUID,
        'trunk'
    ),
    (
        'a0000000-0000-0000-0000-000000000002'::UUID,
        'a0000000-0000-0000-0000-000000000003'::UUID,
        'trunk'
    ),
    (
        'a0000000-0000-0000-0000-000000000003'::UUID,
        'a0000000-0000-0000-0000-000000000004'::UUID,
        'link'
    ),
    (
        'a0000000-0000-0000-0000-000000000001'::UUID,
        'a0000000-0000-0000-0000-000000000005'::UUID,
        'link'
    ) ON CONFLICT (source_id, target_id) DO NOTHING;
COMMIT;