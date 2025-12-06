CREATE TABLE IF NOT EXISTS documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    metadata TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- gemini-embedding-001 default is 3072 dimensions
CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(
    embedding float[3072]
);
