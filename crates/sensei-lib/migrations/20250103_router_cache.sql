CREATE TABLE IF NOT EXISTS router_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query_text TEXT NOT NULL,
    category TEXT NOT NULL,
    enhanced_query TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Vector table for router cache (3072 dimensions for gemini-embedding-001)
CREATE VIRTUAL TABLE IF NOT EXISTS vec_router_cache USING vec0(
    embedding float[3072]
);
