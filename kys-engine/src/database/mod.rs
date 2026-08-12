// JanissaryAsistan - Veri taban\u0131 Katman\u0131 (Supabase / PostgreSQL)
// sqlx ile async PostgreSQL ba\u011flant\u0131s\u0131

use crate::models::*;
use anyhow::Result;
use sqlx::{PgPool, Row};
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};
use std::str::FromStr;
use tracing::info;

// Grafik veri yap\u0131lar\u0131 (main.rs'de de kullan\u0131l\u0131r)
#[derive(serde::Serialize)]
pub struct WeeklyWordPoint {
    pub week: String,
    pub words: i64,
    pub projects: i64,
}

#[derive(serde::Serialize)]
pub struct DailyProjectPoint {
    pub day: String,
    pub count: i64,
}

pub struct ProjectListItem {
    pub id: i64,
    pub filename: String,
    pub total_score: f64,
    pub grade: String,
    pub author: String,
    pub category: String,
    pub status: String,
    pub word_count: i32,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Supabase/PostgreSQL bağlantısı kurar
    /// DATABASE_URL env değişkeninden connection string alır
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Supabase bağlantısı kuruluyor...");
        
        let options = PgConnectOptions::from_str(database_url)?
            .statement_cache_capacity(0); // PgBouncer / Supabase uyuşmazlığını çözer

        let pool = PgPoolOptions::new()
            .max_connections(50)
            .connect_with(options)
            .await
            .map_err(|e| anyhow::anyhow!("Veritabanı bağlantı hatası: {}. DATABASE_URL değişkenini kontrol edin.", e))?;
        
        info!("Supabase bağlantısı başarılı!");
        
        let db = Database { pool };
        // Tablo oluşturma işlemi Supabase arayüzünden migration ile yapıldığı için
        // her açılışta tekrar çalıştırmıyoruz (PgBouncer hatalarını önler).
        
        // Mevcut projelere file_path ve category sütunu ekleyelim
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS file_path TEXT")
            .execute(&db.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS category TEXT")
            .execute(&db.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS manual_rank INTEGER DEFAULT 0")
            .execute(&db.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE scores ADD COLUMN IF NOT EXISTS ai_probability REAL DEFAULT 0.0")
            .execute(&db.pool)
            .await;
        let _ = sqlx::query("ALTER TABLE scores ADD COLUMN IF NOT EXISTS reason TEXT")
            .execute(&db.pool)
            .await;
        
        Ok(db)
    }

    /// Tabloları oluşturur (yoksa)
    async fn create_tables(&self) -> Result<()> {
        let queries = vec![
            "CREATE TABLE IF NOT EXISTS users (
                id              SERIAL PRIMARY KEY,
                name            TEXT NOT NULL,
                email           TEXT UNIQUE NOT NULL,
                password_hash   TEXT NOT NULL,
                is_verified     BOOLEAN DEFAULT FALSE,
                created_at      TIMESTAMP DEFAULT NOW()
            );",
            "CREATE TABLE IF NOT EXISTS email_verifications (
                email           TEXT PRIMARY KEY,
                otp_code        TEXT NOT NULL,
                expires_at      TIMESTAMP NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS projects (
                id          SERIAL PRIMARY KEY,
                filename    TEXT NOT NULL,
                file_type   TEXT NOT NULL,
                word_count  INTEGER,
                language    TEXT,
                author      TEXT DEFAULT 'Bilinmeyen Kullanıcı',
                file_path   TEXT,
                category    TEXT,
                status      TEXT DEFAULT 'Beklemede',
                created_at  TIMESTAMP DEFAULT NOW()
            );",
            "CREATE TABLE IF NOT EXISTS evaluation_categories (
                id              SERIAL PRIMARY KEY,
                name            TEXT NOT NULL,
                criteria_prompt TEXT NOT NULL,
                created_at      TIMESTAMP DEFAULT NOW()
            );",
            "CREATE TABLE IF NOT EXISTS scores (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                category_fit        REAL,
                completeness        REAL,
                reference_quality   REAL,
                technical_depth     REAL,
                originality         REAL,
                ai_probability      REAL DEFAULT 0.0,
                total_score         REAL,
                grade               TEXT,
                reason              TEXT,
                created_at          TIMESTAMP DEFAULT NOW()
            );",
            "CREATE TABLE IF NOT EXISTS similarity_matches (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                title               TEXT,
                url                 TEXT,
                source_type         TEXT,
                similarity_score    REAL,
                matched_keywords    TEXT,
                explanation         TEXT
            );",
            "CREATE TABLE IF NOT EXISTS jury_overrides (
                id                  SERIAL PRIMARY KEY,
                project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                original_total      REAL,
                jury_total          REAL,
                comment             TEXT,
                jury_role           TEXT,
                timestamp           TIMESTAMP DEFAULT NOW()
            );",
            "CREATE TABLE IF NOT EXISTS audit_log (
                id          SERIAL PRIMARY KEY,
                action      TEXT NOT NULL,
                project_id  INTEGER,
                user_role   TEXT,
                details     TEXT,
                timestamp   TIMESTAMP DEFAULT NOW()
            );"
        ];

        for q in queries {
            sqlx::query(q).execute(&self.pool).await?;
        }
        
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS category TEXT").execute(&self.pool).await;
        let _ = sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS manual_rank INTEGER DEFAULT 0").execute(&self.pool).await;

        info!("Tablolar hazır.");
        Ok(())
    }

    pub async fn get_file_path(&self, project_id: i64) -> Result<Option<String>> {
        let row = sqlx::query("SELECT file_path FROM projects WHERE id = $1")
            .bind(project_id as i32)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(r) = row {
            let path: Option<String> = r.get("file_path");
            Ok(path)
        } else {
            Ok(None)
        }
    }

    pub async fn check_project_exists(&self, filename: &str) -> Result<bool> {
        let row = sqlx::query("SELECT id FROM projects WHERE filename = $1")
            .bind(filename)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.is_some())
    }

    /// Yeni proje kaydeder, proje ID döner
    pub async fn save_project(&self, doc: &Document, author: &str, file_path: &str) -> Result<i64> {
        let file_type = format!("{:?}", doc.file_type);
        let language = format!("{:?}", doc.language);
        
        let row = sqlx::query(
            "INSERT INTO projects (filename, file_type, word_count, language, author, file_path) 
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id"
        )
        .bind(&doc.filename)
        .bind(&file_type)
        .bind(doc.word_count as i32)
        .bind(&language)
        .bind(author)
        .bind(file_path)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = row.get("id");
        self.log_action("project_uploaded", Some(id as i64), &doc.filename).await?;
        Ok(id as i64)
    }

    /// Projeye atanan kategoriyi günceller
    pub async fn update_project_category(&self, project_id: i64, category: &str) -> Result<()> {
        sqlx::query("UPDATE projects SET category = $1 WHERE id = $2")
            .bind(category)
            .bind(project_id as i32)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Puanları kaydeder
    pub async fn save_score(&self, project_id: i64, score: &ScoreCard) -> Result<()> {
        sqlx::query(
            "INSERT INTO scores (project_id, category_fit, completeness, reference_quality, 
             technical_depth, originality, ai_probability, total_score, grade, reason)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(project_id as i32)
        .bind(score.category_fit as f32)
        .bind(score.completeness as f32)
        .bind(score.reference_quality as f32)
        .bind(score.technical_depth as f32)
        .bind(score.originality as f32)
        .bind(score.ai_probability as f32)
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

    pub async fn delete_user(&self, email: &str) -> Result<()> {
        sqlx::query("DELETE FROM users WHERE email = $1")
            .bind(email)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn add_evaluation_category(&self, name: &str, criteria_prompt: &str) -> Result<i32> {
        let row = sqlx::query(
            "INSERT INTO evaluation_categories (name, criteria_prompt) VALUES ($1, $2) RETURNING id"
        )
        .bind(name)
        .bind(criteria_prompt)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("id"))
    }

    pub async fn list_evaluation_categories(&self) -> Result<Vec<(i32, String, String)>> {
        let rows = sqlx::query("SELECT id, name, criteria_prompt FROM evaluation_categories ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        
        Ok(rows.iter().map(|r| {
            (r.get("id"), r.get("name"), r.get("criteria_prompt"))
        }).collect())
    }

    pub async fn delete_evaluation_category(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM evaluation_categories WHERE id = $1")
            .bind(id)
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
    pub async fn list_projects(&self) -> Result<Vec<ProjectListItem>> {
        let rows = sqlx::query(
            "SELECT p.id, p.filename, COALESCE(s.total_score::float8, 0.0), COALESCE(s.grade, '?'), 
             COALESCE(p.author, 'Bilinmeyen Kullanıcı'), COALESCE(p.category, 'Genel'), 
             COALESCE(p.status, 'Beklemede'), COALESCE(p.word_count, 0)
             FROM projects p
             LEFT JOIN scores s ON s.project_id = p.id
             ORDER BY CASE WHEN p.manual_rank > 0 THEN p.manual_rank ELSE 999999 END ASC, COALESCE(s.total_score::float8, 0.0) DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|r| {
            ProjectListItem {
                id: r.get::<i32, _>("id") as i64,
                filename: r.get::<String, _>(1),
                total_score: r.get::<f64, _>(2),
                grade: r.get::<String, _>(3),
                author: r.get::<String, _>(4),
                category: r.get::<String, _>(5),
                status: r.get::<String, _>(6),
                word_count: r.get::<i32, _>(7),
            }
        }).collect())
    }

    /// Yeni sıralamayı kaydeder
    pub async fn update_project_ranks(&self, ranks: &[(i64, i32)]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        for (id, rank) in ranks {
            sqlx::query("UPDATE projects SET manual_rank = $1 WHERE id = $2")
                .bind(rank)
                .bind(*id as i32)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Kullanıcı kaydı oluşturur
    pub async fn create_user(&self, name: &str, email: &str, password_hash: &str) -> Result<i64> {
        let row = sqlx::query(
            "INSERT INTO users (name, email, password_hash) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(name)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        let id: i32 = row.get("id");
        Ok(id as i64)
    }

    /// OTP (Doğrulama) kodu kaydeder/günceller
    pub async fn save_otp(&self, email: &str, otp: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO email_verifications (email, otp_code, expires_at) 
             VALUES ($1, $2, NOW() + INTERVAL '15 minutes')
             ON CONFLICT (email) DO UPDATE SET otp_code = EXCLUDED.otp_code, expires_at = EXCLUDED.expires_at"
        )
        .bind(email)
        .bind(otp)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// OTP kodunu doğrular ve hesabı aktifleştirir
    pub async fn verify_otp(&self, email: &str, otp: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT otp_code, expires_at FROM email_verifications WHERE email = $1"
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            let saved_otp: String = r.get("otp_code");
            let expires_at: chrono::NaiveDateTime = r.get("expires_at");
            
            if saved_otp == otp && chrono::Utc::now().naive_utc() < expires_at {
                // Doğrulandı, hesabı aktif et
                sqlx::query("UPDATE users SET is_verified = TRUE WHERE email = $1")
                    .bind(email)
                    .execute(&self.pool)
                    .await?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn get_project_details_full(&self, project_id: i64) -> Result<Option<(ProjectFullRecord, Option<ScoreRecord>, Vec<SimilarityRecord>)>> {
        let p_row = sqlx::query("SELECT id, filename, created_at::text, COALESCE(author, 'Bilinmeyen Kullanıcı') as author, COALESCE(category, 'Genel') as category FROM projects WHERE id = $1")
            .bind(project_id as i32)
            .fetch_optional(&self.pool)
            .await?;
            
        let p_row = match p_row {
            Some(r) => r,
            None => return Ok(None)
        };
        
        let proj = ProjectFullRecord {
            id: p_row.get("id"),
            filename: p_row.get("filename"),
            created_at: p_row.get("created_at"),
            author: p_row.get("author"),
            category: p_row.get("category"),
        };
        
        let s_row = sqlx::query("SELECT category_fit, completeness, reference_quality, technical_depth, originality, ai_probability, total_score, grade FROM scores WHERE project_id = $1")
            .bind(project_id as i32)
            .fetch_optional(&self.pool)
            .await?;
            
        let score = s_row.map(|r| ScoreRecord {
            category_fit: r.get("category_fit"),
            completeness: r.get("completeness"),
            reference_quality: r.get("reference_quality"),
            technical_depth: r.get("technical_depth"),
            originality: r.get("originality"),
            ai_probability: r.try_get("ai_probability").unwrap_or(0.0), // fallback for old data
            total_score: r.get("total_score"),
            grade: r.get("grade"),
        });
        
        let m_rows = sqlx::query("SELECT title, url, source_type, similarity_score FROM similarity_matches WHERE project_id = $1")
            .bind(project_id as i32)
            .fetch_all(&self.pool)
            .await?;
            
        let matches = m_rows.into_iter().map(|r| SimilarityRecord {
            title: r.get("title"),
            url: r.get("url"),
            source_type: r.get("source_type"),
            similarity_score: r.get("similarity_score"),
        }).collect();
        
        Ok(Some((proj, score, matches)))
    }

    /// Grafik verisi: haftalık kelime toplamı + günlük proje sayısı
    pub async fn get_chart_data(&self) -> Result<(Vec<WeeklyWordPoint>, Vec<DailyProjectPoint>)> {
        // Son 8 haftanın kelime toplamı ve proje sayısı
        let weekly_rows = sqlx::query(
            r#"SELECT
                'H' || EXTRACT(WEEK FROM created_at)::int AS week_label,
                COALESCE(SUM(word_count), 0)::bigint AS total_words,
                COUNT(*)::bigint AS total_projects
            FROM projects
            WHERE created_at >= NOW() - INTERVAL '8 weeks'
            GROUP BY EXTRACT(WEEK FROM created_at), week_label
            ORDER BY EXTRACT(WEEK FROM created_at) ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let weekly_words: Vec<WeeklyWordPoint> = weekly_rows.iter().map(|r| WeeklyWordPoint {
            week: r.get::<String, _>("week_label"),
            words: r.get::<i64, _>("total_words"),
            projects: r.get::<i64, _>("total_projects"),
        }).collect();

        // Son 14 g\u00fcn\u00fcn g\u00fcnl\u00fck proje y\u00fckleme say\u0131s\u0131
        let daily_rows = sqlx::query(
            r#"SELECT
                TO_CHAR(created_at, 'DD Mon') AS day_label,
                COUNT(*)::bigint AS total_count
            FROM projects
            WHERE created_at >= NOW() - INTERVAL '14 days'
            GROUP BY DATE_TRUNC('day', created_at), day_label
            ORDER BY DATE_TRUNC('day', created_at) ASC"#
        )
        .fetch_all(&self.pool)
        .await?;

        let daily_projects: Vec<DailyProjectPoint> = daily_rows.iter().map(|r| DailyProjectPoint {
            day: r.get::<String, _>("day_label"),
            count: r.get::<i64, _>("total_count"),
        }).collect();

        Ok((weekly_words, daily_projects))
    }
}

pub struct ProjectFullRecord {
    pub id: i32,
    pub filename: String,
    pub created_at: String,
    pub author: String,
    pub category: String,
}

pub struct ScoreRecord {
    pub category_fit: f32,
    pub completeness: f32,
    pub reference_quality: f32,
    pub technical_depth: f32,
    pub originality: f32,
    pub ai_probability: f32,
    pub total_score: f32,
    pub grade: String,
}

pub struct SimilarityRecord {
    pub title: String,
    pub url: Option<String>,
    pub source_type: String,
    pub similarity_score: f32,
}
