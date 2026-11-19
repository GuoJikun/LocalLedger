-- M1 初始结构：业务表统一带同步元数据，删除一律软删除。

CREATE TABLE IF NOT EXISTS meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS devices (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,
    last_seen_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS accounts (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    type        TEXT NOT NULL DEFAULT 'personal',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    device_id   TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    deleted     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    parent_id   TEXT,
    icon        TEXT,
    color       TEXT,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    device_id   TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    deleted     INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS category_rules (
    id          TEXT PRIMARY KEY,
    keyword     TEXT NOT NULL,
    match_field TEXT NOT NULL,
    category_id TEXT NOT NULL,
    priority    INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    device_id   TEXT NOT NULL,
    version     INTEGER NOT NULL DEFAULT 1,
    deleted     INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS transactions (
    id              TEXT PRIMARY KEY,
    account_id      TEXT NOT NULL,
    source          TEXT NOT NULL,
    external_id     TEXT NOT NULL,
    kind            TEXT NOT NULL,
    amount          INTEGER NOT NULL,
    counterparty    TEXT,
    category_id     TEXT,
    note            TEXT,
    happened_at     TEXT NOT NULL,
    payment_method  TEXT,
    status          TEXT,
    raw_data        TEXT,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL,
    device_id       TEXT NOT NULL,
    version         INTEGER NOT NULL DEFAULT 1,
    deleted         INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (category_id) REFERENCES categories(id),
    UNIQUE (source, external_id)
);

CREATE TABLE IF NOT EXISTS sync_changes (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_type TEXT NOT NULL,
    entity_id   TEXT NOT NULL,
    op          TEXT NOT NULL,
    payload     TEXT,
    created_at  TEXT NOT NULL,
    device_id   TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_happened_at ON transactions(happened_at);
CREATE INDEX IF NOT EXISTS idx_transactions_category ON transactions(category_id);
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_category_rules_category ON category_rules(category_id);
CREATE INDEX IF NOT EXISTS idx_sync_changes_entity ON sync_changes(entity_type, entity_id);

-- 单账本 MVP 的默认账户与常用分类。
INSERT OR IGNORE INTO accounts (id, name, type, created_at, updated_at, device_id, version, deleted)
VALUES ('acc_default', '个人账本', 'personal', '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0);

INSERT OR IGNORE INTO categories (id, name, parent_id, icon, color, sort_order, created_at, updated_at, device_id, version, deleted)
VALUES
    ('cat_food',      '餐饮',    NULL, '🍜', '#f97316', 10, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('cat_transport', '交通',    NULL, '🚌', '#3b82f6', 20, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('cat_shopping',  '购物',    NULL, '🛒', '#a855f7', 30, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('cat_salary',    '工资',    NULL, '💰', '#22c55e', 40, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0),
    ('cat_other',     '其他',    NULL, '📦', '#64748b', 90, '2026-01-01T00:00:00.000Z', '2026-01-01T00:00:00.000Z', 'system', 1, 0);
