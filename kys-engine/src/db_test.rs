// Supabase bağlantı testi - geçici test dosyası
// cargo test db_test -- --nocapture ile çalıştır

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[tokio::test]
    async fn db_connection_test() {
        dotenv::dotenv().ok();
        
        // Port 5432 = session mode, prepared statement destekler
        let url = std::env::var("DIRECT_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .expect("DATABASE_URL veya DIRECT_URL bulunamadı");
        
        println!("Bağlanılıyor: {}...", &url[..url.len().min(55)]);
        
        use sqlx::postgres::PgPoolOptions;
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("Supabase bağlantısı başarısız!");
        
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .expect("Query başarısız!");
        
        assert_eq!(row.0, 1);
        println!("✅ Supabase bağlantısı BAŞARILI!");
    }
}
