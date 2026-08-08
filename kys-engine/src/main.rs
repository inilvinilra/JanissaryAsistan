// KYS Engine - Ana giriş noktası
// Karar Yönetim Sistemi - Proje Analiz ve Araştırma Motoru

use kys_engine::models;
use kys_engine::parser;
use kys_engine::research;
use kys_engine::analysis;
use kys_engine::database;



use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Logging başlat
    tracing_subscriber::fmt::init();
    
    // .env dosyasını yükle (BRAVE_API_KEY burada)
    dotenv::dotenv().ok();

    info!("KYS Engine başlatılıyor...");

    // --- TEST: Basit bir PDF analizi ---
    // Gerçek kullanımda Tauri bu fonksiyonları çağıracak
    
    let test_file = std::env::args().nth(1).unwrap_or_else(|| {
        println!("Kullanım: kys-engine <dosya.pdf|dosya.txt|dosya.md>");
        println!("Örnek: kys-engine proje.pdf");
        std::process::exit(0);
    });

    info!("Dosya analiz ediliyor: {}", test_file);

    // 1. Dosyayı parse et
    let mut document = parser::parse_file(&test_file)?;
    info!("Belge parse edildi: {} kelime, {} anahtar kelime", 
        document.word_count, document.keywords.len());

    println!("\n=== BELGE ANALİZİ ===");
    println!("Dosya: {}", document.filename);
    println!("Kelime sayısı: {}", document.word_count);
    println!("Tespit edilen başlıklar: {:?}", document.headings);
    println!("Anahtar kelimeler: {:?}", &document.keywords[..document.keywords.len().min(10)]);
    println!("Kaynakça var mı: {}", document.has_references);
    println!("Referans sayısı: {}", document.references.len());

    // 2. İnternet araştırması (Federated Search)
    println!("\n=== İNTERNET ARAŞTIRMASI ===");
    let search_results = research::search_related_sources(&document.keywords, "").await?;
    println!("Bulunan kaynak sayısı: {}", search_results.len());

    // 3. Benzerlik analizi
    println!("\n=== BENZERLİK ANALİZİ ===");
    let similarity = analysis::compute_similarity(&document, &search_results);
    
    for result in &similarity.matches {
        println!("- [%{:.1}] {} ({})", 
            result.similarity_score * 100.0,
            result.title,
            result.source_type
        );
    }
    
    println!("\nGenel benzerlik skoru: %{:.1}", similarity.overall_score * 100.0);
    println!("Özgünlük tahmini: {}", similarity.originality_label());

    // 3.5 Taksonomi Analizi
    println!("\n=== TAKSONOMİ VE ALAN UYUMU ===");
    let taxonomy_path = std::path::Path::new("assets/taxonomy.md");
    let (classified_category, category_fit) = if taxonomy_path.exists() {
        let taxonomy = analysis::taxonomy::Taxonomy::load_from_file(taxonomy_path);
        let (cat, fit) = taxonomy.classify(&document.keywords);
        if let Some(ref c) = cat {
            println!("Tespit Edilen Kategori: {}", c);
        } else {
            println!("Kategori tespit edilemedi.");
        }
        (cat, fit)
    } else {
        println!("UYARI: assets/taxonomy.md bulunamadı!");
        (None, 50.0)
    };
    
    document.classified_category = classified_category.clone();

    // 4. Değerlendirme skoru
    println!("\n=== DEĞERLENDİRME ===");
    let score = analysis::score_document(&document, category_fit, classified_category);
    println!("Alan Uyumu:       {:.0}/100", score.category_fit);
    if let Some(cat) = &score.classified_category {
        println!("Kategori:         {}", cat);
    }
    println!("Bölüm Tamlığı:    {:.0}/100", score.completeness);
    println!("Kaynak Kalitesi:  {:.0}/100", score.reference_quality);
    println!("Teknik Derinlik:  {:.0}/100", score.technical_depth);
    println!("─────────────────────");
    println!("TOPLAM PUAN:      {:.1}/100  → {}", score.total(), score.grade());

    Ok(())
}
