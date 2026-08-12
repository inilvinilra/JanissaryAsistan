use sqlx::PgPool;
use sqlx::Executor;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not found");
    println!("Connecting to database...");
    let pool = PgPool::connect(&db_url).await?;
    
    println!("Running migrations...");
    
    let migration_sql = "
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            is_verified BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS projects (
            id SERIAL PRIMARY KEY,
            filename TEXT NOT NULL,
            file_type TEXT NOT NULL,
            word_count INTEGER,
            language TEXT,
            author TEXT DEFAULT 'Bilinmeyen Kullanıcı',
            created_at TIMESTAMP DEFAULT NOW()
        );

        -- Add category and status columns if they didn't exist
        ALTER TABLE projects ADD COLUMN IF NOT EXISTS category TEXT DEFAULT 'Genel';
        ALTER TABLE projects ADD COLUMN IF NOT EXISTS status TEXT DEFAULT 'Beklemede';
        ALTER TABLE projects ADD COLUMN IF NOT EXISTS author TEXT DEFAULT 'Bilinmeyen Kullanıcı';

        CREATE TABLE IF NOT EXISTS evaluation_categories (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            criteria_prompt TEXT NOT NULL,
            created_at TIMESTAMP DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS scores (
            id SERIAL PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            category_fit REAL,
            completeness REAL,
            reference_quality REAL,
            technical_depth REAL,
            originality REAL,
            total_score REAL,
            grade TEXT,
            reason TEXT,
            created_at TIMESTAMP DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS similarity_matches (
            id SERIAL PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            title TEXT,
            url TEXT,
            source_type TEXT,
            similarity_score REAL,
            matched_keywords TEXT,
            explanation TEXT
        );

        CREATE TABLE IF NOT EXISTS jury_overrides (
            id SERIAL PRIMARY KEY,
            project_id INTEGER REFERENCES projects(id) ON DELETE CASCADE,
            original_total REAL,
            jury_total REAL,
            comment TEXT,
            jury_role TEXT,
            timestamp TIMESTAMP DEFAULT NOW()
        );

        CREATE TABLE IF NOT EXISTS audit_log (
            id SERIAL PRIMARY KEY,
            action TEXT NOT NULL,
            project_id INTEGER,
            user_role TEXT,
            details TEXT,
            timestamp TIMESTAMP DEFAULT NOW()
        );

        INSERT INTO evaluation_categories (name, criteria_prompt) 
        SELECT 'Yapay Zeka', 'Projenin yapay zeka ve makine öğrenimi alanındaki derinliğini analiz et.' 
        WHERE NOT EXISTS (SELECT 1 FROM evaluation_categories WHERE name = 'Yapay Zeka');
    ";

    match pool.execute(migration_sql).await {
        Ok(_) => println!("Migrations completed successfully!"),
        Err(e) => println!("Migration error: {}", e),
    }
    
    Ok(())
}
