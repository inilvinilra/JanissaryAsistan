// KYS - Veritabanı Katmanı (Supabase / PostgreSQL)
// sqlx ile async PostgreSQL bağlantısı

use crate::models::*;
use anyhow::Result;
use sqlx::{PgPool, Row};
use tracing::info;

pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Supabase/PostgreSQL bağlantısı kurar
    /// DATABASE_URL env değişkeninden connection string alır
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Supabase bağlantısı kuruluyor...");
        
        let pool = PgPool::connect(database_url).await
            .map_err(|e| anyhow::anyhow!("Veritabanı bağlantı hatası: {}. DATABASE_URL değişkenini kontrol edin.", e))?;
        
        info!("Supabase bağlantısı başarılı!");
        
        let db = Database { pool };
        db.create_tables().await?;
        Ok(db)
    }

    /// Tabloları oluşturur (yoksa)
    async fn create_tables(&self) -> Result<()> {
        sqlx::query("
            CREATE TABLE IF NOT EXISTS projects (
                id          SERIAL PRIMARY KEY,
                filename    TEXT NOT NULL,
                file_type   TEXT NOT NULL,
                word_count  INTEGER,
                language    TEXT,
                created_at  TIMESTAMP DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS scores (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                category_fit        REAL,
                completeness        REAL,
                reference_quality   REAL,
                technical_depth     REAL,
                originality         REAL,
                total_score         REAL,
                grade               TEXT,
                reason              TEXT,
                created_at          TIMESTAMP DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS similarity_matches (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                title               TEXT,
                url                 TEXT,
                source_type         TEXT,
                similarity_score    REAL,
                matched_keywords    TEXT,
                explanation         TEXT
            );

            CREATE TABLE IF NOT EXISTS jury_overrides (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                original_total      REAL,
                jury_total          REAL,
                comment             TEXT,
                jury_role           TEXT,
                timestamp           TIMESTAMP DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS audit_log (
                id          SERIAL PRIMARY KEY,
                action      TEXT NOT NULL,
                project_id  INTEGER,
                user_role   TEXT,
                details     TEXT,
                timestamp   TIMESTAMP DEFAULT NOW()
            );
        ").execute(&self.pool).await?;
        
        info!("Tablolar hazır.");
        Ok(())
    }

    /// Yeni proje kaydeder, proje ID döner
    pub async fn save_project(&self, doc: &Document) -> Result<i64> {
        let file_type = format!("{:?}", doc.file_type);
        let language = format!("{:?}", doc.language);
        
        let row = sqlx::query(
            "INSERT INTO projects (filename, file_type, word_count, language) 
             VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&doc.filename)
        .bind(&file_type)
        .bind(doc.word_count as i32)
        .bind(&language)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = row.get("id");
        self.log_action("project_uploaded", Some(id as i64), &doc.filename).await?;
        Ok(id as i64)
    }

    /// Puanları kaydeder
    pub async fn save_score(&self, project_id: i64, score: &ScoreCard) -> Result<()> {
        sqlx::query(
            "INSERT INTO scores (project_id, category_fit, completeness, reference_quality, 
             technical_depth, originality, total_score, grade, reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(project_id as i32)
        .bind(score.category_fit as f32)
        .bind(score.completeness as f32)
        .bind(score.reference_quality as f32)
        .bind(score.technical_depth as f32)
        .bind(score.originality as f32)
        .bind(score.total() as f32)
        .bind(score.grade())
        .bind(score.reason())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    /// Benzerlik eşleşmelerini kaydeder
    pub async fn save_similarity(&self, project_id: i64, report: &SimilarityReport) -> Result<()> {
        for m in &report.matches {
            let kw = m.matched_keywords.join(",");
            sqlx::query(
                "INSERT INTO similarity_matches 
                 (project_id, title, url, source_type, similarity_score, matched_keywords, explanation)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(project_id as i32)
            .bind(&m.title)
            .bind(&m.url)
            .bind(&m.source_type)
            .bind(m.similarity_score as f32)
            .bind(&kw)
            .bind(&m.explanation)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Jüri puanı değiştirdiğinde kaydeder (audit)
    pub async fn save_jury_override(
        &self,
        project_id: i64,
        original: f64,
        jury_score: f64,
        comment: &str,
        role: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO jury_overrides (project_id, original_total, jury_total, comment, jury_role)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(project_id as i32)
        .bind(original as f32)
        .bind(jury_score as f32)
        .bind(comment)
        .bind(role)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Audit log (KVKK uyumu)
    pub async fn log_action(&self, action: &str, project_id: Option<i64>, details: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_log (action, project_id, details) VALUES ($1, $2, $3)"
        )
        .bind(action)
        .bind(project_id.map(|id| id as i32))
        .bind(details)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Tüm projeleri puanlarıyla listeler (jüri paneli için)
    pub async fn list_projects(&self) -> Result<Vec<(i64, String, f64, String)>> {
        let rows = sqlx::query(
            "SELECT p.id, p.filename, COALESCE(s.total_score::float8, 0.0), COALESCE(s.grade, '?')
             FROM projects p
             LEFT JOIN scores s ON s.project_id = p.id
             ORDER BY s.total_score DESC NULLS LAST"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| {
            let id: i32 = r.get("id");
            let filename: String = r.get("filename");
            let total: f64 = r.get(2);
            let grade: String = r.get(3);
            (id as i64, filename, total, grade)
        }).collect())
    }
}
