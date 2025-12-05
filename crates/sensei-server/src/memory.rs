use anyhow::Result;
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
    pub async fn new(database_url: &str) -> Result<Self> {
        // Register sqlite-vec extension globally for all new connections.
        // SAFETY:
        // 1. `sqlite3_vec_init` is a valid FFI function pointer provided by the `sqlite-vec` crate.
        // 2. `sqlite3_auto_extension` expects a generic function pointer (`void (*)(void)`).
        // 3. `transmute` is used to cast the specific signature of `sqlite3_vec_init` to the generic one expected by `libsqlite3-sys`.
        // This operation is safe as long as the function pointer is valid, which is guaranteed by the crate.
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

    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    // --- Sessions ---

    pub async fn create_session(&self, title: Option<&str>) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        sqlx::query!("INSERT INTO sessions (id, title) VALUES (?, ?)", id, title)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let sessions = sqlx::query_as!(
            Session,
            r#"SELECT id, title, created_at as "created_at: NaiveDateTime" FROM sessions ORDER BY updated_at DESC"#
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(sessions)
    }

    pub async fn get_session(&self, id: &str) -> Result<Session> {
        let session = sqlx::query_as!(
            Session,
            r#"SELECT id, title, created_at as "created_at: NaiveDateTime" FROM sessions WHERE id = ?"#,
            id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(session)
    }

    pub async fn update_session_title(&self, id: &str, title: &str) -> Result<()> {
        sqlx::query!(
            "UPDATE sessions SET title = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            title,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // --- Messages ---

    pub async fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<String> {
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

    pub async fn get_messages(&self, session_id: &str) -> Result<Vec<Message>> {
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

    pub async fn add_document(&self, content: &str, embedding: Vec<f32>) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. Insert content
        // Use query_scalar! for standard table, but handle Option return (though fetch_one should return T)
        // fetch_one returns Result<T, Error>. query_scalar returns Record which might be mapped.
        // Let's use sqlx::query to be safe and consistent with vector part.

        use sqlx::Row;
        let row = sqlx::query("INSERT INTO documents (content) VALUES (?) RETURNING id")
            .bind(content)
            .fetch_one(&mut *tx)
            .await?;

        let id: i64 = row.get("id");

        // 2. Insert vector (using rowid = id)
        // Use sqlx::query (not macro) because vec0 extension is not loaded in cargo check
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
    ) -> Result<Vec<String>> {
        let vector_bytes = f32_vec_to_bytes(&query_embedding);

        // Join vec_items with documents
        // Use sqlx::query (not macro)
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
}

fn f32_vec_to_bytes(v: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(v.len() * 4);
    for f in v {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}
