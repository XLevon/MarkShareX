use sea_orm::*;
use std::sync::{Arc, Mutex};
use tantivy::schema::Value;
use tantivy::schema::{Schema, INDEXED, STORED, TEXT};
use tantivy::{
    collector::TopDocs, directory::MmapDirectory, query::QueryParser, Index, IndexWriter,
    TantivyDocument,
};

/// 在 CJK 字符之间插入空格，使 SimpleTokenizer 能正确切分中/日/韩文字。
/// "MarkFlow博客系统" → "MarkFlow 博 客 系 统"
fn cjk_tokenize(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + text.len() / 3);
    let chars: Vec<char> = text.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        if i > 0 {
            let prev_is_cjk = is_cjk(chars[i - 1]);
            let curr_is_cjk = is_cjk(ch);
            let prev_is_alnum = chars[i - 1].is_alphanumeric();
            let curr_is_alnum = ch.is_alphanumeric();
            // 插入空格：CJK↔CJK, 字母↔CJK, CJK↔字母
            if (prev_is_cjk && curr_is_cjk)
                || (prev_is_alnum && curr_is_cjk)
                || (prev_is_cjk && curr_is_alnum)
            {
                result.push(' ');
            }
        }
        result.push(ch);
    }
    result
}

fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3400}'..='\u{4DBF}' |   // CJK Extension A
        '\u{F900}'..='\u{FAFF}' |   // CJK Compatibility
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}'     // Katakana
    )
}

/// Find the nearest valid UTF-8 char boundary at or before the given byte index.
fn floor_char_boundary(s: &str, index: usize) -> usize {
    if index >= s.len() {
        return s.len();
    }
    // Walk back until we hit a char boundary
    let mut end = index;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    end
}

pub struct SearchEngine {
    pub index: Index,
    writer: Arc<Mutex<IndexWriter<TantivyDocument>>>,
}

impl Clone for SearchEngine {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            writer: Arc::clone(&self.writer),
        }
    }
}

impl SearchEngine {
    pub fn index_document(&self, post_id: u64, title: &str, body: &str) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("搜索索引 writer 锁已损坏"))?;
        let schema = self.index.schema();
        let title_field = schema.get_field("title")?;
        let body_field = schema.get_field("body")?;
        let post_id_field = schema.get_field("post_id")?;

        // Atomically delete any existing document with the same post_id
        // before inserting the new one, so the index always has exactly
        // one document per post.
        let term = tantivy::Term::from_field_u64(post_id_field, post_id);
        writer.delete_term(term);

        let mut doc = TantivyDocument::new();
        let tokenized_title = cjk_tokenize(title);
        let tokenized_body = cjk_tokenize(body);
        tracing::debug!(%post_id, raw_title=%title, tok_title=%tokenized_title, "索引文档");
        doc.add_text(title_field, tokenized_title);
        doc.add_text(body_field, tokenized_body);
        doc.add_u64(post_id_field, post_id);
        writer.add_document(doc)?;
        writer.commit()?;
        Ok(())
    }

    pub fn delete_from_index(&self, post_id: u64) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("搜索索引 writer 锁已损坏"))?;
        let schema = self.index.schema();
        let post_id_field = schema.get_field("post_id")?;
        let term = tantivy::Term::from_field_u64(post_id_field, post_id);
        writer.delete_term(term);
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> anyhow::Result<Vec<u64>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let schema = self.index.schema();
        let title_field = schema.get_field("title")?;
        let body_field = schema.get_field("body")?;
        let post_id_field = schema.get_field("post_id")?;

        let query_parser = QueryParser::for_index(&self.index, vec![title_field, body_field]);
        let tokenized_query = cjk_tokenize(query_str);
        tracing::info!(raw=%query_str, tokenized=%tokenized_query, "搜索查询");
        let query = query_parser.parse_query(&tokenized_query)?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let results: Vec<u64> = top_docs
            .into_iter()
            .filter_map(|(_score, doc_addr)| {
                let doc: TantivyDocument = searcher.doc(doc_addr).ok()?;
                let val = doc.get_first(post_id_field)?;
                val.as_u64()
            })
            .collect();

        Ok(results)
    }

    #[cfg(any(test, debug_assertions))]
    #[doc(hidden)]
    pub fn poison_writer_for_test(&self) {
        let writer = Arc::clone(&self.writer);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = writer.lock().expect("writer must initially be healthy");
            panic!("intentional search writer poison for fault injection");
        }));
    }
}

pub fn init_index(data_dir: &str) -> anyhow::Result<SearchEngine> {
    let mut schema_builder = Schema::builder();
    // 使用默认 TEXT 类型（SimpleTokenizer），配合 cjk_tokenize 预处理
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_u64_field("post_id", INDEXED | STORED);
    let schema = schema_builder.build();

    let index_path = std::path::Path::new(data_dir).join("search_index");
    std::fs::create_dir_all(&index_path)?;
    let dir = MmapDirectory::open(&index_path)?;
    let index = Index::open_or_create(dir, schema)?;
    let writer = index.writer::<TantivyDocument>(15_000_000)?;
    Ok(SearchEngine {
        index,
        writer: Arc::new(Mutex::new(writer)),
    })
}

pub async fn ensure_search_index_consistency(
    engine: &SearchEngine,
    db: &DatabaseConnection,
    post_id: i32,
    mutation_result: anyhow::Result<()>,
) -> Result<(), crate::utils::AppError> {
    let Err(primary_error) = mutation_result else {
        return Ok(());
    };

    tracing::warn!(
        post_id,
        error = %primary_error,
        "搜索索引增量更新失败，尝试从数据库全量重建"
    );
    reindex_all_posts(engine, db)
        .await
        .map_err(|rebuild_error| {
            crate::utils::AppError::Internal(anyhow::anyhow!(
                "文章 {post_id} 的索引增量更新失败: {primary_error}; 全量重建也失败: {rebuild_error}"
            ))
        })
}

/// Rebuild the full-text search index from all published posts in the database.
/// Call this once at startup to populate the index.
pub async fn reindex_all_posts(
    engine: &SearchEngine,
    db: &sea_orm::DatabaseConnection,
) -> anyhow::Result<()> {
    // Clear existing index
    {
        let mut writer = engine
            .writer
            .lock()
            .map_err(|_| anyhow::anyhow!("搜索索引 writer 锁已损坏"))?;
        writer.delete_all_documents()?;
        writer.commit()?;
    }

    // Fetch all published posts via raw SQL (avoids entity import issues)
    let rows = db.query_all(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "SELECT id, title, content FROM posts WHERE status = 'published' AND deleted_at IS NULL".to_string(),
    )).await?;

    let count = rows.len();
    for row in rows {
        let id: i64 = row.try_get_by_index(0).unwrap_or_default();
        let title: String = row.try_get_by_index(1).unwrap_or_default();
        let content: String = row.try_get_by_index(2).unwrap_or_default();
        let body = if content.len() > 50000 {
            // Find nearest valid UTF-8 char boundary at or before byte 50000
            let end = floor_char_boundary(&content, 50000);
            format!("{} {}", title, &content[..end])
        } else {
            format!("{} {}", title, content)
        };
        engine.index_document(id as u64, &title, &body)?;
    }

    tracing::info!("搜索索引重建完成：已索引 {} 篇文章", count);
    Ok(())
}

#[cfg(test)]
mod consistency_tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Database};

    fn temporary_engine() -> anyhow::Result<(tempfile::TempDir, SearchEngine)> {
        let directory = tempfile::tempdir()?;
        let engine = init_index(directory.path().to_str().expect("temporary path is UTF-8"))?;
        Ok((directory, engine))
    }

    fn live_document_count(engine: &SearchEngine) -> anyhow::Result<u64> {
        Ok(engine.index.reader()?.searcher().num_docs())
    }

    #[test]
    fn repeated_updates_leave_one_live_document_and_only_latest_terms() -> anyhow::Result<()> {
        let (_directory, engine) = temporary_engine()?;

        engine.index_document(99, "unrelated-survivor-token", "stable-body-token")?;
        engine.index_document(42, "version-one-token", "first-body-token")?;
        engine.index_document(42, "version-two-token", "second-body-token")?;
        engine.index_document(42, "version-three-token", "latest-body-token")?;

        assert!(engine.search("version-one-token", 10)?.is_empty());
        assert!(engine.search("version-two-token", 10)?.is_empty());
        assert!(engine.search("first-body-token", 10)?.is_empty());
        assert!(engine.search("second-body-token", 10)?.is_empty());
        assert_eq!(engine.search("version-three-token", 10)?, vec![42]);
        assert_eq!(engine.search("latest-body-token", 10)?, vec![42]);
        assert_eq!(engine.search("unrelated-survivor-token", 10)?, vec![99]);
        assert_eq!(engine.search("stable-body-token", 10)?, vec![99]);
        assert_eq!(live_document_count(&engine)?, 2);
        Ok(())
    }

    #[test]
    fn deleting_a_post_preserves_unrelated_live_documents() -> anyhow::Result<()> {
        let (_directory, engine) = temporary_engine()?;
        engine.index_document(7, "delete-me-token", "body")?;
        engine.index_document(8, "keep-me-token", "stable")?;

        assert_eq!(engine.search("delete-me-token", 10)?, vec![7]);
        assert_eq!(engine.search("keep-me-token", 10)?, vec![8]);
        assert_eq!(live_document_count(&engine)?, 2);

        engine.delete_from_index(7)?;

        assert!(engine.search("delete-me-token", 10)?.is_empty());
        assert_eq!(engine.search("keep-me-token", 10)?, vec![8]);
        assert_eq!(live_document_count(&engine)?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn full_reindex_matches_published_non_deleted_rows() -> anyhow::Result<()> {
        let (_directory, engine) = temporary_engine()?;
        let db = Database::connect("sqlite::memory:").await?;
        db.execute_unprepared(
            "CREATE TABLE posts (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                status TEXT NOT NULL,
                deleted_at DATETIME NULL
            );
            INSERT INTO posts VALUES (1, 'published-index-token', 'body', 'published', NULL);
            INSERT INTO posts VALUES (2, 'draft-index-token', 'body', 'draft', NULL);
            INSERT INTO posts VALUES (3, 'deleted-index-token', 'body', 'published', CURRENT_TIMESTAMP);
            INSERT INTO posts VALUES (4, 'second-published-token', 'second-body-token', 'published', NULL);",
        )
        .await?;

        reindex_all_posts(&engine, &db).await?;
        assert_eq!(engine.search("published-index-token", 10)?, vec![1]);
        assert_eq!(engine.search("second-published-token", 10)?, vec![4]);
        assert_eq!(engine.search("second-body-token", 10)?, vec![4]);
        assert!(engine.search("draft-index-token", 10)?.is_empty());
        assert!(engine.search("deleted-index-token", 10)?.is_empty());
        assert_eq!(live_document_count(&engine)?, 2);

        db.execute_unprepared("UPDATE posts SET status = 'draft' WHERE id = 1")
            .await?;
        reindex_all_posts(&engine, &db).await?;
        assert!(engine.search("published-index-token", 10)?.is_empty());
        assert_eq!(engine.search("second-published-token", 10)?, vec![4]);
        assert_eq!(live_document_count(&engine)?, 1);
        Ok(())
    }
}
