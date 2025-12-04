use sensei_server::memory::MemoryStore;
use sqlx::Row;

#[tokio::test]
async fn vector_extension_works() {
    let store = MemoryStore::new("sqlite::memory:").await.unwrap();

    let row = sqlx::query("SELECT vec_version()")
        .fetch_one(store.get_pool())
        .await
        .expect("Failed to query vec_version - extension likely not loaded");

    let version: String = row.get(0);
    println!("sqlite-vec version: {}", version);
    assert!(!version.is_empty());
}
