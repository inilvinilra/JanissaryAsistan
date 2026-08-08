use rand::Rng;
use serde::{Deserialize, Serialize};
use dotenvy::dotenv;

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize)]
struct ResendEmailPayload {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

/// 6 haneli rastgele OTP kodu üretir
pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..999999);
    code.to_string()
}

/// Resend API kullanarak E-posta gönderir
pub async fn send_verification_email(to_email: &str, otp: &str) -> Result<(), String> {
    dotenv().ok();
    
    let api_key = std::env::var("RESEND_API_KEY")
        .map_err(|_| "RESEND_API_KEY bulunamadı. Lütfen .env dosyanızı kontrol edin.".to_string())?;

    let html_content = format!(
        "<h2>KYS Jüri Paneline Hoş Geldiniz!</h2>
         <p>Kayıt işleminizi tamamlamak için doğrulama kodunuz:</p>
         <h1 style='color: #4CAF50;'>{}</h1>
         <p>Bu kod 15 dakika geçerlidir.</p>",
        otp
    );

    let payload = ResendEmailPayload {
        from: "onboarding@resend.dev".to_string(), // Resend Test Email
        to: vec![to_email.to_string()],
        subject: "KYS Dashboard - E-posta Doğrulama Kodu".to_string(),
        html: html_content,
    };

    let client = reqwest::Client::new();
    let res = client.post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("HTTP İsteği Başarısız: {}", e))?;

    if res.status().is_success() {
        Ok(())
    } else {
        let err_text = res.text().await.unwrap_or_default();
        Err(format!("Resend API Hatası: {}", err_text))
    }
}

// TODO: Gerçek veritabanı entegrasyonu tamamlandığında buraya eklenecek.
#[tauri::command]
pub async fn register_user(name: String, email: String, password: String) -> Result<AuthResponse, String> {
    // 1. Şifreyi hashle
    let hashed = bcrypt::hash(password, 10).map_err(|e| e.to_string())?;
    
    // 2. Veritabanına kaydet (Supabase - kys_engine)
    // Şimdilik mock yapıyoruz, Supabase entegrasyonu db.rs'de hazır.
    
    // 3. OTP üret ve maile gönder
    let otp = generate_otp();
    match send_verification_email(&email, &otp).await {
        Ok(_) => Ok(AuthResponse { success: true, message: "Doğrulama kodu gönderildi.".to_string() }),
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn verify_otp(email: String, otp: String) -> Result<AuthResponse, String> {
    // 1. Veritabanından OTP kontrolü yap (Supabase - kys_engine)
    // Şimdilik mock
    if otp.len() == 6 {
        Ok(AuthResponse { success: true, message: "Doğrulama başarılı!".to_string() })
    } else {
        Err("Geçersiz kod".to_string())
    }
}
