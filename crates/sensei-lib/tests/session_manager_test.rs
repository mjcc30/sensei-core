use sensei_lib::memory::MemoryStore;
use uuid::Uuid;

#[tokio::test]
async fn session_crud_works() {
    // 1. Setup (In-Memory DB)
    let store = MemoryStore::new("sqlite::memory:").await.unwrap();
    store.migrate().await.unwrap();

    // 2. Create Session
    let session_id = store.create_session(Some("My Rust Project")).await.unwrap();

    // Check if ID is valid UUID
    assert!(Uuid::parse_str(&session_id).is_ok());

    // 3. List Sessions
    let sessions = store.list_sessions().await.unwrap();
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].title, Some("My Rust Project".to_string())); // Title is Option<String>
    assert_eq!(sessions[0].id, session_id);
    // Check date is roughly now (not year 1970)
    assert!(sessions[0].created_at.and_utc().timestamp() > 1700000000);

    // 4. Rename Session
    store
        .update_session_title(&session_id, "Renamed Project")
        .await
        .unwrap();
    let session = store.get_session(&session_id).await.unwrap();
    assert_eq!(session.title, Some("Renamed Project".to_string()));

    // 5. Delete Session
    store.delete_session(&session_id).await.unwrap();
    let sessions = store.list_sessions().await.unwrap();
    assert!(sessions.is_empty());
}
