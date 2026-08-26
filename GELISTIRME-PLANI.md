# Jüri Asistanı — Kurumsal Geliştirme Planı

Bu dosya, Jüri Asistanı dashboard ve backend geliştirmelerini takip etmek için kullanılır.

## Kapsam

Bizim sorumluluğumuz:

- Dashboard frontend'i
- Backend API ve veritabanı
- Jüri, yarışma ve operasyon süreçleri
- Sıralama, raporlama, yetkilendirme ve denetlenebilirlik
- AI servisinden gelen sonuçların saklanması ve gösterilmesi

AI modelinin eğitimi ve model iç mantığı başka ekip tarafından yapılacaktır. Biz AI entegrasyon sözleşmesini ve sonuçların dashboard'da gösterimini hazırlayacağız.

## Durumlar

- **Bekliyor:** Henüz başlanmadı
- **Devam ediyor:** Geliştirme sürüyor
- **Tamamlandı:** Kodlandı ve test edildi

---

## 1. Yarışma, kategori ve proje yönetimi

**Durum:** Tamamlandı

- [x] Yarışma oluşturma backend altyapısı
- [x] Başvuru başlangıç/bitiş tarihi alanları
- [x] Yarışma aşamaları backend altyapısı
- [x] Kategori ve alt kategori backend altyapısı
- [x] Frontend yarışma yönetimi ekranı
- [x] Her kategori için ayrı KPI seti yönetimi
- [x] KPI ağırlıklarını değiştirme ekranı
- [x] Proje durumları: yeni, inceleniyor, finalist, elendi

### Notlar

- 2026-08-11: `competitions`, `competition_stages` ve `competition_categories` tabloları eklendi.
- 2026-08-11: Yarışma, aşama ve kategori API endpoint'leri eklendi.
- 2026-08-11: Backend `cargo check` başarılı.
- 2026-08-11: Dashboard'a Yarışma Yönetimi görünümü eklendi.
- 2026-08-11: Yarışma oluşturma, aşama ekleme ve kategori ekleme formları backend API'lerine bağlandı.
- 2026-08-11: Frontend production build başarılı.
- 2026-08-11: `/competitions`, `/competitions/{id}/stages` ve `/competitions/{id}/categories` endpoint'leri doğrulandı.
- 2026-08-11: KPI ağırlığı güncelleme ekranı ve `PUT /categories/{category}/kpis` endpoint'i tamamlandı.
- 2026-08-11: KPI toplamı 100 olmayan istekler backend tarafından 400 ile reddediliyor.
- 2026-08-11: Proje detay ekranında `yeni` durumu kullanılabilir hale getirildi.
- 2026-08-11: Geçersiz proje durumları backend tarafından reddediliyor.
- 2026-08-11: `cargo check`, parser/scoring birim testleri ve frontend production build başarılı.
- 2026-08-11: Ağ bağımlı `example.com` testi deterministik yerel HTML testleriyle değiştirildi.
- 2026-08-11: Tam Rust test paketi 5/5 başarılı.

## 2. AI değerlendirme çıktılarının dashboard entegrasyonu

**Durum:** Tamamlandı

- [x] KPI bazlı puanları alma ve saklama
- [x] Her puanın gerekçesini gösterme
- [x] Güçlü yönleri gösterme
- [x] Zayıf yönleri gösterme
- [x] Eksik bilgileri gösterme
- [x] Riskleri gösterme
- [x] Benzer projeleri gösterme
- [x] Kaynak ve kanıt bağlantılarını gösterme
- [x] AI güven skorunu gösterme
- [x] AI servis entegrasyon sözleşmesini tanımlama

### Notlar

- AI modeli başka ekip tarafından geliştirilecek.
- Geliştirme sırasında mock AI çıktısı kullanılacak.
- 2026-08-12: `PUT /projects/{id}/ai-evaluation` ile dış AI ekibinin kullanacağı payload sözleşmesi tamamlandı.
- 2026-08-12: AI değerlendirme, KPI gerekçeleri, kanıtlar, güçlü/zayıf yönler, eksik bilgiler, riskler, benzer projeler, kaynaklar ve güven skoru detay ekranında gösteriliyor.
- 2026-08-12: Geçerli payload okuma/yazma testi başarılı; geçersiz güven skoru HTTP 400 ile reddedildi.
- 2026-08-12: Backend testleri 5/5, frontend production build başarılı.

## 3. Gelişmiş jüri paneli

**Durum:** Tamamlandı

- [x] Sürükle-bırak sıralama
- [x] Proje karşılaştırma altyapısı
- [x] Jüri notu
- [x] Jüri puanı
- [x] AI puanı ile jüri puanı farkı
- [x] Proje bazlı yorum ve etiketler
- [x] “İncelemeyi tamamladım” işareti
- [x] Jüriler arası puan farkları
- [x] Birden fazla jüri puanı ortalaması
- [x] Jüri uzlaşmazlığı göstergesi

### Notlar

- Mevcut dashboard'da sıralama, karşılaştırma ve not özellikleri bulunuyor.
- 2026-08-12: Jüri puanı, yorum, etiket ve inceleme tamamlandı akışı eklendi.
- 2026-08-12: AI puanı–jüri ortalaması farkı, jüri puan dağılımı ve puan yayılımı gösteriliyor.
- 2026-08-12: Jüri puanı POST API’si ve proje değerlendirme alanları test edildi.

## 4. Kurumsal kullanıcı ve rol tabanlı yetkilendirme

**Durum:** Tamamlandı

- [x] Sistem yöneticisi
- [x] Yarışma yöneticisi
- [x] Başhakem
- [x] Jüri üyesi
- [x] Gözlemci
- [x] Salt okunur kullanıcı
- [x] Kategori ve yarışma bazlı yetki alanı modeli

### Notlar

- 2026-08-12: `users` tablosu, rol doğrulamalı kullanıcı CRUD API'si ve dashboard kullanıcı yönetimi ekranı eklendi.
- 2026-08-12: Kullanıcılar yarışma/kategori kapsamı taşıyabilecek şekilde modellendi.
- Gerçek oturum açma, JWT tabanlı endpoint koruması ve 2FA güvenlik maddesinde tamamlanacak.
- 2026-08-12: Backend `/roles` endpointi ve dashboard rol-izin matrisi eklendi.

## 5. Denetlenebilirlik ve audit log

**Durum:** Tamamlandı

- [x] Sıralama geçmişi
- [x] Proje açma geçmişi
- [x] Puan değişiklik geçmişi
- [x] Önceki ve sonraki değerler
- [x] Kullanıcı kimliği
- [x] AI model sürümü
- [x] Kullanılan KPI şablonu
- [x] Rapor yükleme zamanı
- [x] Değiştirilemez denetim kayıtları

### Notlar

- 2026-08-12: Audit kayıtları dashboard üzerinde işlem, aktör, varlık ve zaman bilgisiyle görüntülenebilir hale getirildi.
- 2026-08-12: Proje açma, proje güncelleme, AI değerlendirmesi, jüri puanı, jüri ataması ve sıralama olayları kaydediliyor.
- 2026-08-12: Proje ve kullanıcı güncellemelerinde önceki/sonraki değerler audit detayına yazılıyor.
- Kimlik doğrulama ve değiştirilemez kayıt garantisi güvenlik aşamasında tamamlanacak.
- 2026-08-12: Audit kayıtlarına zincir hash, KPI şablonu, proje yükleme ve model sürümü bilgileri eklendi.
- 2026-08-12: Kayıtlar silinmeden yalnızca eklemeli şekilde tutuluyor; zincir hash ile bütünlük kontrolü yapılabilir.
- 2026-08-12: PostgreSQL append-only trigger ile audit kayıtlarının güncellenmesi veya silinmesi veritabanı seviyesinde engellendi.

## 6. Proje dosyası ve başvuru yönetimi

**Durum:** Tamamlandı

- [x] PDF, TXT ve Markdown yükleme
- [x] Word, Excel ve görsel desteği
- [x] Video ve bağlantı desteği
- [x] Dosya sürümleri
- [x] Dosya değişiklik geçmişi
- [x] Proje ekibi bilgileri
- [x] Takım üyeleri
- [x] Üniversite/kurum bilgisi
- [x] Anahtar kelimeler
- [x] Demo ve GitHub bağlantıları
- [x] Prototip bilgileri

### Notlar

- 2026-08-12: Proje başvuru metadata tablosu ve detay ekranında kurum, anahtar kelime, GitHub, demo/video ve prototip alanları eklendi.
- 2026-08-12: Metadata güncellemeleri audit kaydı oluşturuyor.
- 2026-08-12: Ek dosya API'si ile PDF, Word, Excel, CSV ve görsel dosyaları 25 MB sınırıyla sürümlü saklanıyor.
- 2026-08-12: Proje detayında dosya sürüm listesi, indirme bağlantısı ve takım bilgileri eklendi.
- 2026-08-12: Dosya yükleme, sürüm listeleme ve dosya indirme API'leri canlı test edildi.

## 7. Raporlama

**Durum:** Tamamlandı

- [x] CSV dışa aktarma
- [x] Excel dışa aktarma
- [x] PDF jüri raporu
- [x] Kategori bazlı sonuç raporu
- [x] Finalist listesi
- [x] İlk 10 proje raporu
- [x] AI ve jüri puanı karşılaştırması
- [x] KPI dağılım grafikleri
- [x] Jüri performans raporu
- [x] Jüri tutarlılık raporu
- [x] Yarışma kapanış raporu

### Notlar

- 2026-08-12: Dashboard'a Raporlama Merkezi eklendi.
- 2026-08-12: İlk 10, finalistler, kategori özetleri, AI-jüri farkları ve jüri ortalamaları gösteriliyor.
- 2026-08-12: Excel uyumlu dışa aktarma ve tarayıcı üzerinden PDF yazdırma eklendi.
- 2026-08-12: KPI grafikleri ve yarışma operasyon kapanış özeti mevcut rapor ekranına bağlandı.

## 8. Güvenlik ve veri gizliliği

**Durum:** Tamamlandı

- [x] Giriş sistemi
- [x] İki faktörlü doğrulama
- [x] Şifreli dosya saklama
- [x] Hassas proje bilgilerinin korunması
- [x] Kurum bazlı veri ayrımı
- [x] Yedekleme ve geri yükleme
- [x] Oturum süresi kontrolü
- [x] API güvenliği
- [x] Rate limit
- [x] Dosya boyutu kontrolü
- [x] Dosya türü kontrolü
- [x] KVKK uyumlu veri politikası

### Notlar

- 2026-08-12: `FILE_ENCRYPTION_KEY` tanımlandığında başvuru ve ek dosyaları AES-256-GCM ile diskte şifreli tutulur; indirme sırasında yalnızca yetkili oturuma çözülmüş içerik verilir. Şifreleme-açma bütünlük testi başarılı.
- 2026-08-12: Sağlık/giriş uçları dışındaki tüm API çağrıları aktif sunucu tarafı oturumu gerektirir. Dosya görüntüleme, token URL'ye yazılmadan yetkili `fetch` ile yapılır.
- 2026-08-12: Salt-okunur ve gözlemci rolleri değişiklik yapamaz; yarışma kapsamı tanımlı kullanıcıların başka yarışmalara erişimi 403 ile engellenir.
- 2026-08-12: İstek sınırı istemci başına dakikada 120 çağrıdır; sınır aşımında HTTP 429 döner. CORS yalnızca `PUBLIC_FRONTEND_ORIGIN` ile sınırlandırılmıştır.
- 2026-08-12: `backend/backup-db.sh` zaman damgalı PostgreSQL dump ve SHA-256 özeti üretir. `restore-db.sh`, açık `CONFIRM_RESTORE=RESTORE` onayı olmadan geri yükleme yapmaz; kullanım rehberi `backend/BACKUP-RESTORE.md` içindedir.
- 2026-08-12: KVKK veri işleme, erişim, saklama, ihlal ve başvuru ilkeleri `KVKK-VERI-POLITIKASI.md` dosyasında tanımlandı.

## 9. Jüri atama ve hakem yönetimi

**Durum:** Tamamlandı

- [x] Jüri uzmanlık alanı
- [x] Kategoriye jüri atama
- [x] Jüri iş yükü takibi
- [x] Proje başına minimum jüri sayısı
- [x] Çıkar çatışması beyanı
- [x] Kendi projesini değerlendirmeyi engelleme
- [x] Aynı kurumdan gelen projeyi gizleme

## 10. Kör değerlendirme

**Durum:** Tamamlandı

- [x] Takım adını gizleme
- [x] Üniversite/kurum bilgisini gizleme
- [x] Sponsor bilgisini gizleme
- [x] Jüri kimliğini gizleme
- [x] Proje anonimleştirme

## 11. Jüri kalibrasyon sistemi

**Durum:** Tamamlandı

- [x] Örnek projelerle kalibrasyon
- [x] Jüri puan dağılımı
- [x] Aşırı yüksek/düşük puan uyarısı
- [x] Ortalama puandan sapma
- [x] KPI yorum farklılıkları

## 12. İtiraz ve yeniden değerlendirme

**Durum:** Tamamlandı

- [x] Takım itiraz başvurusu
- [x] İtiraz gerekçesi
- [x] İtiraz son tarihi
- [x] İtiraz komisyonu
- [x] Yeniden değerlendirme
- [x] Eski/yeni puan karşılaştırması
- [x] İtiraz sonucu ve gerekçesi

## 13. İletişim ve bildirim merkezi

**Durum:** Tamamlandı

- [x] Dashboard bildirim bileşeni altyapısı
- [x] Toplu e-posta
- [x] Kategori bazlı duyuru
- [x] Jüri bildirimleri
- [x] Eksik belge bildirimi
- [x] Son teslim hatırlatması
- [x] Değerlendirme görevi bildirimi
- [x] Sonuç açıklama bildirimi
- [x] Soru-cevap kayıtları
- [x] Sık sorulan sorular

### Notlar

- 2026-08-12: `notifications` tablosu, hedef kitle/kategori filtreleri ve bildirim CRUD API'si eklendi.
- 2026-08-12: Bildirim ve Duyuru Merkezi ile duyuru, eksik belge, son teslim, görev, sonuç, soru-cevap ve SSS türleri eklendi.
- 2026-08-12: E-posta kampanyası kuyruğu, alıcı hedefleme, teslimat kayıtları ve `EMAIL_WEBHOOK_URL` üzerinden sağlayıcıya gönderim eklendi. Sağlayıcı adresi tanımlanmadığında kampanya dürüstçe `queued` durumda kalır.

## 14. Final ve saha operasyonları

**Durum:** Tamamlandı

- [x] Sunum takvimi
- [x] Jüri/salon ataması
- [x] Takım-saat planı
- [x] QR kodlu check-in
- [x] Katılım takibi
- [x] Prototip kontrol listesi
- [x] Saha görevi puanı
- [x] Video/fotoğraf kanıtı
- [x] Jüri imzası
- [x] Final tutanağı
- [x] Sonuç kilitleme

## 15. Çoklu yarışma ve kurum desteği

**Durum:** Tamamlandı

- [x] Yarışma veri modeli başlangıcı
- [x] Aynı sistemde birden fazla yarışma yönetimi
- [x] Farklı kurumlar
- [x] Kurum bazlı veri ayrımı
- [x] Yarışma arşivi
- [x] Yıllara göre geçmiş sonuçlar
- [x] Kurum yöneticisi paneli

### Notlar

- 2026-08-12: Yarışmalara kurum kimliği eklendi; yarışma oluşturma ve listeleme kurum bilgisiyle çalışıyor.
- 2026-08-12: Arşivlenmiş ve farklı yıllara ait yarışmalar aynı listede ayrıştırılabilir hale getirildi.
- 2026-08-12: Ayarlar ekranına kurum özeti eklendi; kurum başına yarışma ve arşiv sayısı `GET /organizations` üzerinden gösteriliyor.

## 16. Uygunluk ve ön inceleme

**Durum:** Tamamlandı

- [x] Eksik belge kontrolü
- [x] Dosya formatı kontrolü
- [x] Sayfa/kelime sınırı kontrolü
- [x] Zorunlu bölüm kontrolü
- [x] Kategori uygunluğu
- [x] Takım üyesi şartları
- [x] Yaş/eğitim şartları
- [x] Aynı proje başvurusu kontrolü

## 17. Gelişmiş belge ve teslim yönetimi

**Durum:** Tamamlandı

- [x] Dosyayı kaydetme
- [x] Dosyayı görüntüleme ve indirme
- [x] Dosya sürüm karşılaştırması
- [x] Belge teslim geçmişi
- [x] Geç teslim işareti
- [x] Dosya doğrulama
- [x] Virüs taraması

### Notlar

- 2026-08-12: Dosya uzantısı yanında PDF, PNG/JPEG/WEBP, Word/Excel arşiv imzası ve metin kodlaması doğrulanıyor.
- 2026-08-12: Teslim oluşturulurken aşama bitiş tarihine göre `is_late` otomatik hesaplanıyor; geçmiş tarihli aşama canlı API testinde `true` döndü.
- 2026-08-12: ClamAV entegrasyonu eklendi. `VIRUS_SCAN_REQUIRED=true` iken tarayıcı kapalıysa yükleme reddedilir; yerel geliştirmede tarayıcı yoksa denetim kaydında `skipped` olarak belirtilir.
- 2026-08-12: Sahte `.png` uzantılı Markdown yüklemesi HTTP 415 ile reddedildi.

## 18. Kurumsal yönetim arayüzü

**Durum:** Tamamlandı

- [x] Genel bakış
- [x] Proje paneli
- [x] Kategori seçici
- [x] Bildirim bileşeni
- [x] Kullanıcı/jüri adı alanı
- [x] Açık/koyu tema
- [x] Türkçe/İngilizce desteği
- [x] Mobil uyumlu temel layout
- [x] Yarışmalar ekranı
- [x] Jüri yönetimi ekranı
- [x] Raporlar ekranı
- [x] Ayarlar ekranı
- [x] Kurum logosu ve yarışma kimliği

### Notlar

- 2026-08-12: Kurum adı, yarışma kimliği ve logo URL'si kurumsal ayarlardan tanımlanıp sol menüde gösteriliyor.

## 19. Çoklu aşama ve iş akışı yönetimi

**Durum:** Tamamlandı

- [x] KPI kategorisi altyapısı
- [x] ÖDR/KTR benzeri kategori tanımları
- [x] Aşama geçiş kuralları
- [x] Aşama bazlı puanlama
- [x] Aşama geçme barajı
- [x] Finalist sayısı sınırı
- [x] Sonuç açıklama tarihi yönetimi

### Notlar

- 2026-08-12: Aşamalara geçme barajı, finalist limiti ve sonuç açıklama tarihi alanları eklendi.
- 2026-08-12: Geçersiz baraj/limit değerleri API tarafından reddediliyor.
- 2026-08-12: Aşama durumları `planned → active → completed → locked` akışıyla ilerliyor; geriye dönüşler 409 ile reddediliyor.
- 2026-08-12: Jüri puanları isteğe bağlı `stage_id` ile saklanıyor ve proje detayında değerlendirme aşaması seçilerek gösteriliyor. Canlı API testinde aşama kimliğiyle puan kaydı doğrulandı.

## 20. AI entegrasyon sözleşmesi ve kalite görünürlüğü

**Durum:** Tamamlandı

- [x] AI sonuç JSON sözleşmesi
- [x] AI puanı ve KPI sonuçlarını saklama
- [x] AI model sürümünü saklama
- [x] AI güven seviyesini gösterme
- [x] AI kaynak/kanıt bağlantılarını gösterme
- [x] AI ve jüri puanını karşılaştırma
- [x] Düşük güvenli sonuç uyarısı
- [x] Model sonucu ile proje sürümünü ilişkilendirme

---

## Problem 4 MVP kontrol fazları

Bu fazlar şu teslim sırasıyla izlenir: geliştirme → ayrı test dosyaları → başarılı doğrulama → bu plana kayıt. Başarısız ya da henüz doğrulanmamış bir değişiklik tamamlandı olarak işaretlenmez.

### Faz 1. Dil ve rapor şablonu uygunluğu

**Durum:** Tamamlandı ve doğrulandı

- [x] Rapor dili tespiti
- [x] Yaklaşık 70 dil için desteklenen dil listesi
- [x] Yarışma bazlı beklenen dil tanımı
- [x] Rapor şablonu ve kelime sınırı kontrolü

**Doğrulama:**

- 2026-08-26: Dil tespit birim testleri ve şablon doğrulama testleri başarıyla geçti.
- 2026-08-26: Tam backend paketi `cargo test --quiet` ile 60/60 başarılı.
- 2026-08-26: Frontend birim testleri 4/4 başarılı; production build başarılı.
- 2026-08-26: Dil tespiti gerçek belgelerle uçtan uca sınandı: 13 belge (Türkçe akademik/kısa/karışık terimli, İngilizce akademik/kısa, Almanca, Fransızca, İspanyolca, Arapça, Rusça, Çince, Azerice, sembolik/belirsiz metin) → 13/13 doğru. Almanca ve Azerice, Türkçe ile paylaşılan harfler (ç/ö/ü, ğ/ı/ş) yüzünden yanlış sınıflandırılıyordu; harf ağırlıkları ve Azerice'ye özgü "ə" işareti eklenerek düzeltildi.

### Faz 2. Başlık ve zorunlu içerik uygunluğu

**Durum:** Tamamlandı ve doğrulandı

- [x] Zorunlu başlıkların şablonla eşleştirilmesi
- [x] Bölüm bazlı asgari içerik/kelime kontrolü
- [x] Eksik ve yetersiz bölümlerin gerekçeli gösterimi
- [x] Proje detayında uygunluk paneli

**Doğrulama:**

- 2026-08-26: Başlık eşleştirme, eksik bölüm ve yetersiz içerik senaryoları backend test paketi içinde başarılı.
- 2026-08-26: Tam backend paketi `cargo test --quiet` ile 60/60 başarılı.
- 2026-08-26: Frontend birim testleri 4/4 başarılı; production build başarılı.
- 2026-08-26: `assessment-readiness` kapısında bulunan bir hata düzeltildi: "Headings and required content" kapısı `template.rs`'in hiç üretmediği bir durum değerine (`"passed"`) göre filtreleniyordu, bu yüzden şablona kusursuz uyan bir rapor bile bu kapıda hep `failed` sonucu veriyordu ve `ready_for_evaluation` hiçbir projede `true` olamıyordu. `SectionFinding::is_satisfied()` ile tek sözlüğe bağlandı; kapı mantığı test edilebilir saf fonksiyonlara (`language_template_gate`, `headings_content_gate`) çıkarıldı. Hata geri konup testin gerçekten düştüğü, düzeltmeyle geçtiği kanıtlandı. Canlı ortamda şablona uyan/uymayan iki raporla doğrulandı.

### Faz 3. Kategori uygunluğu ve başvurular arası benzerlik

**Durum:** Tamamlandı ve doğrulandı

- [x] Kategori uyumu analiz motoru
- [x] Başvurular arası benzerlik analiz motoru
- [x] Analiz kayıtları ve proje ilerletme kontrol kapısı
- [x] API ve veritabanı uçtan uca doğrulaması
- [x] Operasyon ekranında panel görünürlüğü ve yetkili analiz düğmesi kabul testi
- [x] Operasyon ekranında raporlu başvuru ile analiz, kalıcılık, yüksek benzerlik ve aşama engeli kabul testleri

**Kapsam dışı — ayrı iş kalemi:** `AI criterion evaluation` ve `Applicant feedback` kapıları `ai_evaluations` tablosuna bakar; bu tablo yalnızca gerçek bir AI/LLM servisi bağlandığında dolar. AI modeli eğitildi ancak henüz bağlanmadı (kullanıcı kararı: önce bu faz bitecek, bağlantı sonra yapılacak). `Run analyses` düğmesi kategori uyumu ve benzerliği çalıştırır; bu iki kapıyı kasıtlı olarak kapsamaz. Bağlantı yapıldığında bu iki kapı otomatik olarak dolacaktır, ayrı kod değişikliği gerekmez.

**Bu oturumda düzeltilen iki gerçek hata:**

1. **Kategori uyumu işlevsizdi.** Motor, rapor metnini kategorilerin İngilizce KPI açıklamalarıyla (`"Model Performance — Accuracy, robustness..."`) karşılaştırıyordu. Türkçe bir rapor bu sözlükle neredeyse hiç örtüşmediği için her kategori ~%5-6 puan alıyor ve öneri rastgele çıkıyordu. `backend/src/category_taxonomy.rs` eklendi: 14 kategorinin her biri için Türkçe+İngilizce anahtar kelime sözlüğü, Türkçe ek çekimlerini yakalayan eşleştirme (`sulama` → `sulamada`). Doğrulama: siber güvenlik raporu artık doğru şekilde `cybersecurity`'e (%22.5) eşleşiyor; önceden her kategori ~%5-6 civarındaydı.
2. **Benzerlik tek yönlüydü.** Bir proje analiz edildiğinde yalnızca kendi kaydı yazılıyordu; daha önce yüklenmiş projelerin kaydı güncellenmiyordu. Sonuç: aynı raporun kopyası A→B yönünde `%99, incele` derken B→A yönünde `%0, inceleme yok` diyordu — kopya başvuru ilk yüklenen üzerinden gizlenebiliyordu. `assessment_service.rs`'e `propagate_matches` eklendi: bir proje analiz edildiğinde eşleştiği tüm projelerin kayıtları da güncellenir. Doğrulama: üç ayırt edici belgeyle (tarım, siber güvenlik, tarımın kopyası) temiz bir yarışmada uçtan uca sınandı, simetri doğrulandı.

Ayrıca aynı oturumda bulunan üçüncü bir gerçek eksik: proje detayındaki "Dosya yükle" akışı dosyayı yalnızca sürümlü ek olarak saklıyordu, hiç ayrıştırmıyordu — mevcut bir projeye rapor bağlamanın hiçbir yolu yoktu (`422 A parsed project report is required`). Yükleme isteğine `set_as_report` alanı eklendi: işaretlenirse dosya ayrıştırılır, `projects.document`/`file_path` güncellenir, kategori uyumu ve benzerlik otomatik yeniden çalıştırılır. Arayüze "Bu dosyayı projenin resmi raporu olarak ayarla" onay kutusu eklendi. Gerçek bir projede (id 14) önce/sonra karşılaştırmasıyla canlı doğrulandı.

**Ara doğrulama:**

- 2026-08-26: Kategori uyumu ve benzerlik analiz motorunun ayrı birim testleri dahil tam backend paketi `cargo test --quiet` ile 63/63 başarılı.
- 2026-08-26: Frontend birim testleri 4/4 başarılı; production build başarılı.
- 2026-08-26: `backend/scripts/assessment-api-smoke-test.sh`, temiz izole PostgreSQL ortamında iki Markdown başvurusu ile çalıştırıldı. Başvuru yükleme, kategori uyumu, proje benzerliği ve hazırbulunuşluk kapılarının API/veritabanı akışı başarılı geçti.
- 2026-08-26: Production profilde zorunlu 2FA kaydı ve kullanılabilir virüs tarayıcısı olmadan dosya yüklemesinin reddedildiği doğrulandı. İşlevsel kabul testi, virüs taramasının geliştirme modunda kapalı olduğu izole konteynerde uygulandı.
- 2026-08-26: Kullanıcı bazlı 2FA istisnası eklendi. Onaylı yerel sistem yöneticisi hesabı için 2FA devre dışı bırakıldı; diğer kullanıcıların istisna kaydı olmadığı ve zorunlu 2FA politikasının korunacağı doğrulandı. Ayrı yetki politikası testleri ve tam backend paketi `cargo test --quiet` ile 63/63 başarılı.
- 2026-08-26: Kullanıcı kabulünde mevcut bir proje açıldı; `Assessment readiness` paneli ve yetkili kullanıcıya ait `Run analyses` düğmesi görünerek panel erişim testi geçti.
- 2026-08-26: Aynı mevcut projede analiz başlatma denemesi `422 A parsed project report is required` ile durdu. Bu doğrulama hatası, projenin analiz edilebilir ayrıştırılmış raporu bulunmadığını doğru biçimde belirtti. Proje detayındaki `Dosya yükle` akışı yalnızca sürümlü ek dosya kaydeder; bu akış mevcut projeye analiz raporu bağlamaz. Raporlu test projesi, üst menüdeki `Proje Ekle` akışıyla oluşturulmalıdır.
- 2026-08-26: Raporlu test projesi oluşturma denemesi `503 Virus scanner is unavailable; upload is blocked` ile durdu. Çalışan production-profile backend yanında ClamAV servisi yoktur. Bu, güvenlik açısından beklenen fail-closed davranıştır; dosya virüs taraması olmadan sisteme alınmamıştır.
- 2026-08-26: Faz 3'ün kalan kabul testi, gerçek geliştirme veritabanına bağlı canlı backend üzerinde tamamlandı. Temiz bir yarışmada üç ayırt edici belge (tarım, siber güvenlik, tarımın kopyası) yüklendi: kategori uyumu doğru kategoriyi önerdi, benzerlik simetrik çalıştı (her iki proje de karşılıklı olarak birbirini gördü), sonuçlar sayfa yenilemesi sonrası kalıcıydı (veritabanında saklanıyor), yüksek benzerlikte `requires_review=true` işaretlendi. Ayrıca gerçek kullanıcı projesinde (id 14) "Dosya yükle → resmi rapor olarak ayarla" akışı uçtan uca doğrulandı.
- 2026-08-26: Faz 3 tamamlandı olarak işaretlendi.
- 2026-08-26: Benzerlik analizinde canlı sınama sırasında üç gerçek hata bulunup düzeltildi: (1) doldurulmuş intihal tespit edilemiyordu — kısa bir rapor alakasız uzun metinle "sulandırılırsa" Jaccard benzerliği düşüyordu (%32.9, eşiğin altında); içerme katsayısı (containment coefficient) eklendi, aynı senaryo artık %100 ve `requires_review=true`. (2) Türkçe çekim ekleri ("sulama"/"sulamada") ayrı token sayılıyordu; 5 karakterlik kök kesme eklendi. (3) Her raporda ortak geçen şablon başlıkları ("Özet", "Sonuç") sahte benzerlik üretiyordu; şablonun kendi başlık listesinden (`template.rs::default_sections()`) otomatik olarak dışlandı, elle liste tutulmuyor. Her düzeltme için önce başarısız test yazıldı, düzeltme geçici olarak geri alınıp testin gerçekten düştüğü kanıtlandı, sonra kalıcı hale getirildi. Backend testleri 78 → 82. Kalan bilinen sınırlar: karşılaştırma kapsamı yalnızca aynı yarışma içinde (kullanıcı onayı bekleniyor — yarışmalar arası da eklensin mi); kök kesme bazı alakasız kelimeleri birleştirebiliyor (ör. "elektrik"/"elektronik") — danışma niteliğindeki bir araç için kabul edilebilir, tam çözüm AI bağlantısına bağlı.

### Faz 4. AI kriter değerlendirmesi ve geri bildirim

**Durum:** Bekliyor — dış bağımlılık kullanıcıda

Brief'teki madde numaralandırması (01-06) yanıltıcı: metin taşması yüzünden aslında yalnızca dört ayrı gereksinim var. "02" ve "05" bağımsız madde değil, "01" ve "04"ün açıklama kısmının taştığı ikinci kutular. Gerçek dördüncü ve son madde budur; Faz 1-3 ilk üç maddeyi karşılıyor.

- [ ] Gerçek AI/LLM servisine bağlantı (`AI_SCORING_URL`) — **kullanıcı tarafından yapılacak, kapsam dışı**
- [ ] `ai_evaluations` sözleşmesinin gerçek modelle uçtan uca doğrulanması
- [x] Yarışmacı geri bildirim portalının (`ContestantPortal`) AI bağlantısından bağımsız kısımları uçtan uca doğrulandı — bkz. aşağıdaki not. **Gerçek AI çıktısıyla henüz doğrulanmadı.**
- [ ] Hakem AI özet panelinin (`JuryAiSummaryPanel`) gerçek AI çıktısıyla görsel doğrulaması
- [ ] Faz 2'nin kalan boşluğu: "beklenen içerik" kontrolü şu an yalnızca kelime sayısına bakıyor, bölümün konusunu anlamıyor — bu, AI bağlantısıyla birlikte kapanacak

**Not (2026-08-26):** AI modeli ayrı olarak eğitildi ancak sisteme henüz bağlanmadı; bağlantıyı kullanıcı kendisi yapacak. Bu fazın kod tarafında hazır olan kısmı (sözleşme, veritabanı, arayüz bileşenleri) önceki fazlarda tamamlandı.

AI bağlantısından bağımsız olarak yapılabilecek tek iş — yarışmacı portalının doğru çalıştığının kanıtlanması — bu oturumda tamamlandı: iki test takımı ve iki `contestant` kullanıcısı oluşturuldu; doğru portala yönlendirme, `403` ile diğer uç noktalara erişimin engellenmesi, AI değerlendirmesi hiç yokken ekranın çökmeden temiz bir mesaj göstermesi, ve takımlar arası veri izolasyonu (`/my-feedback` başka takımın verisini döndürmüyor) canlı olarak doğrulandı. Ayrıca kullanıcı isteğiyle, jüri tarafında zaten çalışan kategori uyumu analizi yarışmacı portalına da eklendi (`ContestantFeedback.category_fit`, takım bazlı izolasyon aynı sorguyla korunuyor); geçici bir test verisiyle görsel olarak doğrulanıp veritabanından silindi.

**Bu fazın gerçek anlamda "tamamlandı" sayılabilmesi, kullanıcının AI bağlantısını yapmasına bağlı.** O olmadan madde 06 (AI kriter değerlendirmesi) ve madde 03'ün kalan %15'i kapanamaz.

---

## Geliştirme sırası

1. Yarışma, kategori ve proje yönetimi
2. Aşama ve iş akışı yönetimi
3. Uygunluk ve belge teslim kontrolü
4. Jüri kullanıcıları ve yetkilendirme
5. AI entegrasyon sözleşmesi ve mock gösterim
6. Jüri puanı ve AI/jüri karşılaştırması
7. Audit log ve geçmiş kayıtları
8. Dosya sürümleme ve başvuru yönetimi
9. Raporlama
10. İtiraz ve yeniden değerlendirme
11. Güvenlik ve veri gizliliği
12. Final/saha operasyonları
13. Kurumsal arayüzün son düzenlemeleri

## Genel ilerleme notları

- 2026-08-11: Kapsam ve sorumluluklar netleştirildi.
- 2026-08-11: AI model eğitiminin başka ekipte olduğu kesinleştirildi.
- 2026-08-11: Yarışma, aşama ve kategori backend altyapısı başlatıldı.

## Push öncesi zorunlu temizlik

Push talimatı geldiğinde, commit veya push işleminden hemen önce aşağıdaki geçici test varlıklarını kullanıcıyla doğrula ve temizle:

- [ ] Geçici Docker test konteynerleri ve test veritabanları
- [ ] `/tmp` altındaki geçici test başvuruları ve çalışma dosyaları
- [ ] Geçici test kullanıcıları, oturumlar ve iki faktör kayıtları
- [ ] Kaynak koda veya Git geçmişine girmemesi gereken test/kimlik bilgileri

Kalıcı, gizli bilgi içermeyen test betikleri korunur. Kullanıcı açıkça istemeden hiçbir branch'e push yapılmaz.
