use std::sync::Arc;
use std::sync::OnceLock;

use futures::StreamExt;
use lancedb::arrow::arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator,
    RecordBatchReader as LanceRecordBatchReader, StringArray,
};
use lancedb::arrow::arrow_schema::{ArrowError, DataType, Field, Schema};
use lancedb::query::{ExecutableQuery, QueryBase};

use super::db::AppDb;
use crate::error::CntrlError;

const EMBED_DIM: i32 = 768;
const LANCE_TABLE: &str = "task_vectors";
const OLLAMA_EMBED_URL: &str = "http://localhost:11434/api/embeddings";
const EMBED_MODEL: &str = "nomic-embed-text";

static LANCE_DB: OnceLock<lancedb::Connection> = OnceLock::new();

pub fn init_lance_db(data_dir: &str) {
    let path = format!("{data_dir}/lancedb");
    tauri::async_runtime::spawn(async move {
        match lancedb::connect(&path).execute().await {
            Ok(conn) => {
                let _ = LANCE_DB.set(conn);
                eprintln!("[recall] LanceDB initialised at {path}");
            }
            Err(e) => {
                eprintln!("[recall] LanceDB init failed (using SQLite fallback): {e}");
            }
        }
    });
}

#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub intent_raw: String,
    pub intent_type: String,
    pub result: Option<String>,
    pub created_at: String,
}

#[derive(serde::Deserialize)]
struct OllamaEmbedResponse {
    embedding: Vec<f32>,
}

async fn get_embedding(text: &str) -> Option<Vec<f32>> {
    let client = reqwest::Client::new();
    let resp = client
        .post(OLLAMA_EMBED_URL)
        .json(&serde_json::json!({
            "model": EMBED_MODEL,
            "prompt": text
        }))
        .timeout(std::time::Duration::from_secs(8))
        .send()
        .await
        .ok()?;

    let body: OllamaEmbedResponse = resp.json().await.ok()?;
    if body.embedding.is_empty() {
        None
    } else {
        Some(body.embedding)
    }
}

fn task_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("intent_raw", DataType::Utf8, false),
        Field::new("intent_type", DataType::Utf8, false),
        Field::new("result", DataType::Utf8, true),
        Field::new("created_at", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                EMBED_DIM,
            ),
            false,
        ),
    ]))
}

fn pad_to_dim(mut v: Vec<f32>) -> Vec<f32> {
    v.resize(EMBED_DIM as usize, 0.0);
    v
}

fn make_batch(
    schema: Arc<Schema>,
    id: &str,
    intent_raw: &str,
    intent_type: &str,
    result: Option<&str>,
    created_at: &str,
    embedding: Vec<f32>,
) -> Result<RecordBatch, ArrowError> {
    let values = Float32Array::from(pad_to_dim(embedding));
    let vector_field = Arc::new(Field::new("item", DataType::Float32, true));
    let vector_col = FixedSizeListArray::new(vector_field, EMBED_DIM, Arc::new(values), None);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec![id])),
            Arc::new(StringArray::from(vec![intent_raw])),
            Arc::new(StringArray::from(vec![intent_type])),
            Arc::new(StringArray::from(vec![result])),
            Arc::new(StringArray::from(vec![created_at])),
            Arc::new(vector_col),
        ],
    )
}

async fn upsert_to_lancedb(
    conn: &lancedb::Connection,
    id: &str,
    intent_raw: &str,
    intent_type: &str,
    result: Option<&str>,
    created_at: &str,
    embedding: Vec<f32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schema = task_schema();
    let batch = make_batch(
        schema.clone(),
        id,
        intent_raw,
        intent_type,
        result,
        created_at,
        embedding,
    )?;

    let make_reader = |b: RecordBatch| -> Box<dyn LanceRecordBatchReader + Send> {
        let schema = b.schema();
        let iter = vec![Ok(b)].into_iter();
        Box::new(RecordBatchIterator::new(iter, schema))
    };

    match conn.open_table(LANCE_TABLE).execute().await {
        Ok(tbl) => {
            tbl.add(make_reader(batch)).execute().await?;
        }
        Err(_) => {
            conn.create_table(LANCE_TABLE, make_reader(batch))
                .execute()
                .await?;
        }
    }
    Ok(())
}

async fn search_lancedb(
    conn: &lancedb::Connection,
    query_vec: Vec<f32>,
    limit: usize,
) -> Result<Vec<MemoryEntry>, Box<dyn std::error::Error + Send + Sync>> {
    let tbl = conn.open_table(LANCE_TABLE).execute().await?;
    let padded = pad_to_dim(query_vec);

    let mut stream = tbl
        .query()
        .nearest_to(padded.as_slice())?
        .limit(limit)
        .execute()
        .await?;

    let mut entries = Vec::new();
    while let Some(batch_result) = stream.next().await {
        let batch = batch_result?;
        let ir_col = batch
            .column_by_name("intent_raw")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());
        let it_col = batch
            .column_by_name("intent_type")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());
        let res_col = batch
            .column_by_name("result")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());
        let ca_col = batch
            .column_by_name("created_at")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>());

        if let (Some(ir), Some(it), Some(res), Some(ca)) = (ir_col, it_col, res_col, ca_col) {
            for i in 0..batch.num_rows() {
                entries.push(MemoryEntry {
                    intent_raw: ir.value(i).to_string(),
                    intent_type: it.value(i).to_string(),
                    result: if res.is_null(i) {
                        None
                    } else {
                        Some(res.value(i).to_string())
                    },
                    created_at: ca.value(i).to_string(),
                });
            }
        }
    }
    Ok(entries)
}

pub async fn find_relevant_context(
    db: &AppDb,
    intent: &str,
    limit: u32,
) -> Result<Vec<MemoryEntry>, CntrlError> {
    if intent.trim().is_empty() {
        return Ok(vec![]);
    }

    if let Some(conn) = LANCE_DB.get() {
        if let Some(embedding) = get_embedding(intent).await {
            match search_lancedb(conn, embedding, limit as usize).await {
                Ok(entries) if !entries.is_empty() => return Ok(entries),
                Ok(_) => {}
                Err(e) => {
                    eprintln!("[recall] LanceDB search failed, falling back to SQLite: {e}");
                }
            }
        }
    }

    sqlite_keyword_search(db, intent, limit).await
}

pub async fn save_task(
    db: &AppDb,
    id: &str,
    intent_raw: &str,
    intent_type: &str,
    slots_json: &str,
    status: &str,
    result: Option<&str>,
) -> Result<(), CntrlError> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO task_history(id, intent_raw, intent_type, slots, status, result, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(id) DO UPDATE SET status=excluded.status, result=excluded.result, updated_at=excluded.updated_at",
    )
    .bind(id)
    .bind(intent_raw)
    .bind(intent_type)
    .bind(slots_json)
    .bind(status)
    .bind(result)
    .bind(&now)
    .bind(&now)
    .execute(db)
    .await?;

    if status == "done" {
        let id = id.to_string();
        let intent_raw = intent_raw.to_string();
        let intent_type = intent_type.to_string();
        let result_owned = result.map(str::to_string);
        let now_clone = now.clone();

        tauri::async_runtime::spawn(async move {
            if let Some(conn) = LANCE_DB.get() {
                if let Some(embedding) = get_embedding(&intent_raw).await {
                    let res = upsert_to_lancedb(
                        conn,
                        &id,
                        &intent_raw,
                        &intent_type,
                        result_owned.as_deref(),
                        &now_clone,
                        embedding,
                    )
                    .await;
                    if let Err(e) = res {
                        eprintln!("[recall] Failed to index task in LanceDB: {e}");
                    }
                }
            }
        });
    }

    Ok(())
}

async fn sqlite_keyword_search(
    db: &AppDb,
    intent: &str,
    limit: u32,
) -> Result<Vec<MemoryEntry>, CntrlError> {
    let tokens: Vec<&str> = intent.split_whitespace().filter(|w| w.len() >= 4).collect();

    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<String> = tokens
        .iter()
        .map(|_| "intent_raw LIKE ?".to_string())
        .collect();
    let where_clause = placeholders.join(" OR ");
    let sql = format!(
        "SELECT intent_raw, intent_type, result, created_at
         FROM task_history
         WHERE status = 'done' AND ({where_clause})
         ORDER BY created_at DESC
         LIMIT {limit}"
    );

    let mut query = sqlx::query_as::<_, (String, String, Option<String>, String)>(&sql);
    for token in &tokens {
        query = query.bind(format!("%{token}%"));
    }

    let rows = query.fetch_all(db).await?;

    Ok(rows
        .into_iter()
        .map(
            |(intent_raw, intent_type, result, created_at)| MemoryEntry {
                intent_raw,
                intent_type,
                result,
                created_at,
            },
        )
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::memory::db::open_in_memory;

    async fn seed_task(db: &AppDb, id: &str, raw: &str, intent_type: &str, result: &str) {
        save_task(db, id, raw, intent_type, "{}", "done", Some(result))
            .await
            .expect("seed must succeed");
    }

    #[tokio::test]
    async fn recall_finds_matching_task() {
        let db = open_in_memory().await.expect("DB must open");
        seed_task(
            &db,
            "task-1",
            "find me a recipe for lasagne",
            "search",
            "Found recipe",
        )
        .await;

        let results = find_relevant_context(&db, "recipe for pasta", 10)
            .await
            .expect("recall must succeed");

        assert!(!results.is_empty(), "should find the seeded lasagne task");
        assert!(results[0].intent_raw.contains("lasagne"));
    }

    #[tokio::test]
    async fn recall_returns_empty_for_no_match() {
        let db = open_in_memory().await.expect("DB must open");
        seed_task(&db, "task-2", "navigate to github", "navigate", "done").await;

        let results = find_relevant_context(&db, "bitcoin price today", 10)
            .await
            .expect("recall must succeed");

        assert!(
            results.is_empty(),
            "should return empty for no matching keyword"
        );
    }

    #[tokio::test]
    async fn recall_returns_empty_for_empty_intent() {
        let db = open_in_memory().await.expect("DB must open");
        let results = find_relevant_context(&db, "", 10)
            .await
            .expect("recall must succeed");
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn recall_respects_limit() {
        let db = open_in_memory().await.expect("DB must open");
        for i in 0..10 {
            seed_task(
                &db,
                &format!("task-limit-{i}"),
                &format!("search for item number {i}"),
                "search",
                "done",
            )
            .await;
        }

        let results = find_relevant_context(&db, "search item", 3)
            .await
            .expect("recall must succeed");

        assert!(results.len() <= 3, "recall must respect the limit");
    }

    #[tokio::test]
    async fn sqlite_fallback_matches_keywords() {
        let db = open_in_memory().await.expect("DB must open");
        seed_task(
            &db,
            "task-vector-1",
            "search for bitcoin price prediction",
            "search",
            "Result returned",
        )
        .await;

        let results = sqlite_keyword_search(&db, "bitcoin price", 10)
            .await
            .expect("fallback must succeed");

        assert!(!results.is_empty());
        assert!(results[0].intent_raw.contains("bitcoin"));
    }
}
