use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{ConnectOptions, Executor};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Barrier;

// Target: 396,610 ops/sec
const TARGET_OPS: u64 = 400_000;
const BATCH_SIZE: usize = 10_000;
const TOTAL_OPS: usize = 1_000_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "🚀 Starting SQLite Benchmark (Target: {} ops/sec)",
        TARGET_OPS
    );

    // 1. Configure Optimized SQLite
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal) // Crucial for performance vs safety trade-off
        .page_size(4096)
        .disable_statement_logging();

    let pool = SqlitePoolOptions::new()
        .max_connections(10) // Parallelism
        .connect_with(opts)
        .await?;

    // 2. Setup Schema
    pool.execute("CREATE TABLE bench (id INTEGER PRIMARY KEY, val TEXT)")
        .await?;

    // 3. Benchmark: INSERT (Write)
    let start = Instant::now();

    // We use batching because individual inserts are network/syscall bound, not DB bound.
    // Real-world high-throughput systems use batching.
    let mut tasks = Vec::new();
    let _barrier = Arc::new(Barrier::new(11)); // 10 workers + main

    for _i in 0..10 {
        let pool = pool.clone();
        let chunk_size = TOTAL_OPS / 10;

        tasks.push(tokio::spawn(async move {
            for _j in 0..(chunk_size / BATCH_SIZE) {
                // Construct batch insert
                let mut query = String::with_capacity(BATCH_SIZE * 20);
                query.push_str("INSERT INTO bench (val) VALUES ");
                for k in 0..BATCH_SIZE {
                    if k > 0 {
                        query.push(',');
                    }
                    query.push_str("('test_value')");
                }

                sqlx::query(&query).execute(&pool).await.unwrap();
            }
        }));
    }

    for t in tasks {
        t.await?;
    }

    let duration = start.elapsed();
    let ops_sec = TOTAL_OPS as f64 / duration.as_secs_f64();

    println!("✅ Writes Completed: {} ops in {:.2?}", TOTAL_OPS, duration);
    println!("⚡ Write Throughput: {:.0} ops/sec", ops_sec);

    if ops_sec > TARGET_OPS as f64 {
        println!("🏆 TARGET BEATEN! Sensei is Enterprise Ready.");
    } else {
        println!(
            "⚠️  Target missed by {:.0} ops/sec. Needs tuning.",
            TARGET_OPS as f64 - ops_sec
        );
    }

    // 4. Benchmark: SELECT (Read)
    let start_read = Instant::now();
    // Simple count is optimized in SQLite
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bench")
        .fetch_one(&pool)
        .await?;

    println!("📊 Count: {}", row.0);

    // High concurrency reads (Point lookups)
    let read_ops = 200_000;
    let mut read_tasks = Vec::new();
    for _ in 0..10 {
        let pool = pool.clone();
        read_tasks.push(tokio::spawn(async move {
            for _ in 0..(read_ops / 10) {
                sqlx::query("SELECT val FROM bench WHERE id = 1")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            }
        }));
    }

    for t in read_tasks {
        t.await?;
    }

    let read_duration = start_read.elapsed();
    let read_ops_sec = read_ops as f64 / read_duration.as_secs_f64();
    println!(
        "⚡ Read Throughput: {:.0} ops/sec (Point Lookups)",
        read_ops_sec
    );

    Ok(())
}
