use crate::errors::SenseiError;
use chrono::NaiveDateTime;
use libsqlite3_sys::sqlite3_auto_extension;
use serde::{Deserialize, Serialize};
use sqlite_vec::sqlite3_vec_init;
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub created_at: NaiveDateTime,
}

#[derive(Clone)]
pub struct MemoryStore {
    pool: SqlitePool,
}

impl MemoryStore {
    pub async fn new(database_url: &str) -> Result<Self, SenseiError> {
        // Register sqlite-vec extension globally for all new connections.
        unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
        }

        let pool = SqlitePool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn migrate(&self) -> Result<(), SenseiError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    // --- Sessions ---

    pub async fn create_session(&self, title: Option<&str>) -> Result<String, SenseiError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!("INSERT INTO sessions (id, title) VALUES (?, ?)", id, title)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>, SenseiError> {
        let sessions = sqlx::query_as!(
            Session,
            r#"SELECT id, title, created_at as "created_at: NaiveDateTime" FROM sessions ORDER BY updated_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session, SenseiError> {
        let session = sqlx::query_as!(
            Session,
            r#"SELECT id, title, created_at as "created_at: NaiveDateTime" FROM sessions WHERE id = ?"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(session)
    }

    pub async fn update_session_title(&self, id: &str, title: &str) -> Result<(), SenseiError> {
        sqlx::query!(
            "UPDATE sessions SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            title,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<(), SenseiError> {
        sqlx::query!("DELETE FROM sessions WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Messages ---

    pub async fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<String, SenseiError> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!(
            "INSERT INTO messages (id, session_id, role, content) VALUES (?, ?, ?, ?)",
            id,
            session_id,
            role,
            content
        )
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, SenseiError> {
        let messages = sqlx::query_as!(
            Message,
            r#"SELECT id, session_id, role, content, created_at as "created_at: NaiveDateTime" FROM messages WHERE session_id = ? ORDER BY created_at ASC"#,
            session_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }

    // --- RAG / Vectors ---

    pub async fn add_document(
        &self,
        content: &str,
        embedding: Vec<f32>,
    ) -> Result<(), SenseiError> {
        let mut tx = self.pool.begin().await?;

        use sqlx::Row;
        let row = sqlx::query("INSERT INTO documents (content) VALUES (?) RETURNING id")
            .bind(content)
            .fetch_one(&mut *tx)
            .await?;

        let id: i64 = row.get("id");

        let vector_bytes = f32_vec_to_bytes(&embedding);
        sqlx::query("INSERT INTO vec_items (rowid, embedding) VALUES (?, ?)")
            .bind(id)
            .bind(vector_bytes)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn search_documents(
        &self,
        query_embedding: Vec<f32>,
        limit: i64,
    ) -> Result<Vec<String>, SenseiError> {
        let vector_bytes = f32_vec_to_bytes(&query_embedding);

        let rows = sqlx::query(
            r#"
            SELECT d.content, v.distance
            FROM vec_items v
            JOIN documents d ON v.rowid = d.id
            WHERE v.embedding MATCH ? AND k = ?
            ORDER BY v.distance
            "#,
        )
        .bind(vector_bytes)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        use sqlx::Row;
        let results: Vec<String> = rows.iter().map(|row| row.get("content")).collect();
        Ok(results)
    }

    // MCP Support methods
    pub async fn list_documents(&self) -> Result<Vec<(i64, String)>, SenseiError> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT id, substr(content, 1, 50) as snippet FROM documents ORDER BY created_at DESC LIMIT 50")
            .fetch_all(&self.pool)
            .await?;

        let results: Vec<(i64, String)> = rows
            .iter()
            .map(|row| (row.get("id"), row.get::<String, _>("snippet")))
            .collect();
        Ok(results)
    }

    pub async fn get_document(&self, id: i64) -> Result<String, SenseiError> {
        use sqlx::Row;
        let row = sqlx::query("SELECT content FROM documents WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("content"))
    }

    // --- Router Semantic Cache ---

    pub async fn add_router_cache(
        &self,
        query_text: &str,
        category: &str,
        enhanced_query: &str,
        embedding: Vec<f32>,
    ) -> Result<(), SenseiError> {
        let mut tx = self.pool.begin().await?;

        use sqlx::Row;
        let row = sqlx::query(
            "INSERT INTO router_cache (query_text, category, enhanced_query) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(query_text)
        .bind(category)
        .bind(enhanced_query)
        .fetch_one(&mut *tx)
        .await?;

        let id: i64 = row.get("id");
        let vector_bytes = f32_vec_to_bytes(&embedding);

        sqlx::query("INSERT INTO vec_router_cache (rowid, embedding) VALUES (?, ?)")
            .bind(id)
            .bind(vector_bytes)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn search_router_cache(
        &self,
        query_embedding: Vec<f32>,
        similarity_threshold: f32,
    ) -> Result<Option<(String, String)>, SenseiError> {
        let vector_bytes = f32_vec_to_bytes(&query_embedding);

        // Search for nearest neighbor
        let row = sqlx::query(
            r#"
            SELECT c.category, c.enhanced_query, v.distance
            FROM vec_router_cache v
            JOIN router_cache c ON v.rowid = c.id
            WHERE v.embedding MATCH ? AND k = 1
            ORDER BY v.distance ASC
            "#,
        )
        .bind(vector_bytes)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let distance: f32 = r.get("distance");

            if distance < similarity_threshold {
                let category: String = r.get("category");
                let enhanced: String = r.get("enhanced_query");
                return Ok(Some((category, enhanced)));
            }
        }

        Ok(None)
    }

    pub async fn update_router_cache_category(
        &self,
        query_embedding: Vec<f32>,
        new_category: &str,
    ) -> Result<bool, SenseiError> {
        let vector_bytes = f32_vec_to_bytes(&query_embedding);

        // Find nearest neighbor to update
        let row = sqlx::query(
            r#"
            SELECT rowid, distance
            FROM vec_router_cache
            WHERE embedding MATCH ? AND k = 1
            ORDER BY distance ASC
            "#,
        )
        .bind(vector_bytes)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let distance: f32 = r.get("distance");
            let id: i64 = r.get("rowid");

            // Only update if very similar (avoid correcting unrelated queries)
            if distance < 0.05 {
                sqlx::query("UPDATE router_cache SET category = ? WHERE id = ?")
                    .bind(new_category)
                    .bind(id)
                    .execute(&self.pool)
                    .await?;
                return Ok(true);
            }
        }

        Ok(false)
    }
}

fn f32_vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}
