use sensei_lib::memory::MemoryStore;

#[tokio::test]
async fn memory_rag_workflow() {
    let store = MemoryStore::new("sqlite::memory:").await.unwrap();
    store.migrate().await.unwrap();

    // 1. Add Document
    let content = "The secret code is 12345.";
    let embedding = vec![0.0; 3072]; // Mock embedding

    store
        .add_document(content, embedding.clone())
        .await
        .unwrap();

    // 2. List Documents (MCP feature)
    let docs = store.list_documents().await.unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].1, content); // Tuple (id, content)

    // 3. Get Document by ID
    let doc_id = docs[0].0;
    let fetched_content = store.get_document(doc_id).await.unwrap();
    assert_eq!(fetched_content, content);

    // 4. Search (KNN)
    let results = store.search_documents(embedding, 1).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], content);
}
