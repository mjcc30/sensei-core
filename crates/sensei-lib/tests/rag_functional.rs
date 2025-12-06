use sensei_lib::memory::MemoryStore;
use sqlx::Row;

// Helper to convert &[f32] to raw bytes (Little Endian) for sqlite-vec
fn f32_vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

#[tokio::test]
async fn rag_vector_search_works() {
    let store = MemoryStore::new("sqlite::memory:").await.unwrap();

    // 1. Create Virtual Table
    // vec0 is the module name for sqlite-vec
    sqlx::query("CREATE VIRTUAL TABLE vec_items USING vec0(embedding float[2])")
        .execute(store.get_pool())
        .await
        .expect("Failed to create virtual table - check if extension is loaded");

    // 2. Insert Vectors
    // Item 1: [1.0, 0.0] (Horizontal)
    // Item 2: [0.0, 1.0] (Vertical)
    let vec1 = f32_vec_to_bytes(&[1.0, 0.0]);
    let vec2 = f32_vec_to_bytes(&[0.0, 1.0]);

    sqlx::query("INSERT INTO vec_items(rowid, embedding) VALUES (1, ?), (2, ?)")
        .bind(vec1)
        .bind(vec2)
        .execute(store.get_pool())
        .await
        .expect("Failed to insert vectors");

    // 3. Search Nearest Neighbor for [0.9, 0.1]
    // Should be close to Item 1
    // Note: sqlite-vec syntax for KNN uses 'MATCH' and 'k = N' predicate
    let query_vec = f32_vec_to_bytes(&[0.9, 0.1]);

    let row = sqlx::query(
        "SELECT rowid, distance FROM vec_items WHERE embedding MATCH ? AND k = 1 ORDER BY distance",
    )
    .bind(query_vec)
    .fetch_one(store.get_pool())
    .await
    .expect("Failed to search");

    let id: i64 = row.get("rowid");
    // let dist: f32 = row.get("distance");

    assert_eq!(id, 1, "Should match item 1 (Horizontal)");
}
