-- 默认自动分类规则：导入时按 priority 从高到低匹配关键词。
INSERT OR IGNORE INTO category_rules
    (id, keyword, match_field, category_id, priority, created_at, updated_at, device_id, version, deleted)
VALUES
    ('rule_salary_1', '工资',     'counterparty', 'cat_salary',    100, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_salary_2', '工资',     'note',         'cat_salary',     90, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_food_1',   '餐饮',     'category',     'cat_food',       80, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_food_2',   '美团',     'counterparty', 'cat_food',       70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_food_3',   '饿了么',   'counterparty', 'cat_food',       70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_food_4',   '餐厅',     'counterparty', 'cat_food',       60, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_transport_1', '滴滴',   'counterparty', 'cat_transport',  80, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_transport_2', '地铁',   'counterparty', 'cat_transport',  70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_transport_3', '公交',   'counterparty', 'cat_transport',  70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_shopping_1', '淘宝',    'counterparty', 'cat_shopping',   70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_shopping_2', '京东',    'counterparty', 'cat_shopping',   70, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('rule_shopping_3', '超市',    'counterparty', 'cat_shopping',   60, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0);
