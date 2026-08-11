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

**Durum:** Kısmen tamamlandı

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

**Durum:** Kısmen tamamlandı

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

**Durum:** Bekliyor

- [ ] Giriş sistemi
- [ ] İki faktörlü doğrulama
- [ ] Şifreli dosya saklama
- [ ] Hassas proje bilgilerinin korunması
- [ ] Kurum bazlı veri ayrımı
- [ ] Yedekleme ve geri yükleme
- [ ] Oturum süresi kontrolü
- [ ] API güvenliği
- [ ] Rate limit
- [ ] Dosya boyutu kontrolü
- [ ] Dosya türü kontrolü
- [ ] KVKK uyumlu veri politikası

## 9. Jüri atama ve hakem yönetimi

**Durum:** Bekliyor

- [ ] Jüri uzmanlık alanı
- [ ] Kategoriye jüri atama
- [ ] Jüri iş yükü takibi
- [ ] Proje başına minimum jüri sayısı
- [ ] Çıkar çatışması beyanı
- [ ] Kendi projesini değerlendirmeyi engelleme
- [ ] Aynı kurumdan gelen projeyi gizleme

## 10. Kör değerlendirme

**Durum:** Bekliyor

- [ ] Takım adını gizleme
- [ ] Üniversite/kurum bilgisini gizleme
- [ ] Sponsor bilgisini gizleme
- [ ] Jüri kimliğini gizleme
- [ ] Proje anonimleştirme

## 11. Jüri kalibrasyon sistemi

**Durum:** Bekliyor

- [ ] Örnek projelerle kalibrasyon
- [ ] Jüri puan dağılımı
- [ ] Aşırı yüksek/düşük puan uyarısı
- [ ] Ortalama puandan sapma
- [ ] KPI yorum farklılıkları

## 12. İtiraz ve yeniden değerlendirme

**Durum:** Bekliyor

- [ ] Takım itiraz başvurusu
- [ ] İtiraz gerekçesi
- [ ] İtiraz son tarihi
- [ ] İtiraz komisyonu
- [ ] Yeniden değerlendirme
- [ ] Eski/yeni puan karşılaştırması
- [ ] İtiraz sonucu ve gerekçesi

## 13. İletişim ve bildirim merkezi

**Durum:** Kısmen mevcut

- [x] Dashboard bildirim bileşeni altyapısı
- [ ] Toplu e-posta
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
- Toplu e-posta gönderimi için harici e-posta sağlayıcısı entegrasyonu bekliyor; uygulama içi bildirim akışı hazır.

## 14. Final ve saha operasyonları

**Durum:** Bekliyor

- [ ] Sunum takvimi
- [ ] Jüri/salon ataması
- [ ] Takım-saat planı
- [ ] QR kodlu check-in
- [ ] Katılım takibi
- [ ] Prototip kontrol listesi
- [ ] Saha görevi puanı
- [ ] Video/fotoğraf kanıtı
- [ ] Jüri imzası
- [ ] Final tutanağı
- [ ] Sonuç kilitleme

## 15. Çoklu yarışma ve kurum desteği

**Durum:** Kısmen mevcut

- [x] Yarışma veri modeli başlangıcı
- [ ] Aynı sistemde birden fazla yarışma yönetimi
- [x] Farklı kurumlar
- [x] Kurum bazlı veri ayrımı
- [x] Yarışma arşivi
- [x] Yıllara göre geçmiş sonuçlar
- [ ] Kurum yöneticisi paneli

### Notlar

- 2026-08-12: Yarışmalara kurum kimliği eklendi; yarışma oluşturma ve listeleme kurum bilgisiyle çalışıyor.
- 2026-08-12: Arşivlenmiş ve farklı yıllara ait yarışmalar aynı listede ayrıştırılabilir hale getirildi.

## 16. Uygunluk ve ön inceleme

**Durum:** Bekliyor

- [ ] Eksik belge kontrolü
- [ ] Dosya formatı kontrolü
- [ ] Sayfa/kelime sınırı kontrolü
- [ ] Zorunlu bölüm kontrolü
- [ ] Kategori uygunluğu
- [ ] Takım üyesi şartları
- [ ] Yaş/eğitim şartları
- [ ] Aynı proje başvurusu kontrolü

## 17. Gelişmiş belge ve teslim yönetimi

**Durum:** Kısmen mevcut

- [x] Dosyayı kaydetme
- [x] Dosyayı görüntüleme ve indirme
- [x] Dosya sürüm karşılaştırması
- [x] Belge teslim geçmişi
- [ ] Geç teslim işareti
- [x] Dosya doğrulama
- [ ] Virüs taraması

### Notlar

- 2026-08-12: Dosya uzantısı yanında PDF, PNG/JPEG/WEBP, Word/Excel arşiv imzası ve metin kodlaması doğrulanıyor.
- Virüs taraması için harici güvenlik tarayıcısı entegrasyonu bekliyor.

## 18. Kurumsal yönetim arayüzü

**Durum:** Kısmen mevcut

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

**Durum:** Kısmen mevcut

- [x] KPI kategorisi altyapısı
- [x] ÖDR/KTR benzeri kategori tanımları
- [x] Aşama geçiş kuralları
- [ ] Aşama bazlı puanlama
- [x] Aşama geçme barajı
- [x] Finalist sayısı sınırı
- [x] Sonuç açıklama tarihi yönetimi

### Notlar

- 2026-08-12: Aşamalara geçme barajı, finalist limiti ve sonuç açıklama tarihi alanları eklendi.
- 2026-08-12: Geçersiz baraj/limit değerleri API tarafından reddediliyor.
- 2026-08-12: Aşama durumları `planned → active → completed → locked` akışıyla ilerliyor; geriye dönüşler 409 ile reddediliyor.

## 20. AI entegrasyon sözleşmesi ve kalite görünürlüğü

**Durum:** Tamamlandı

- [x] AI sonuç JSON sözleşmesi
- [x] AI puanı ve KPI sonuçlarını saklama
- [x] AI model sürümünü saklama
- [x] AI güven seviyesini gösterme
- [x] AI kaynak/kanıt bağlantılarını gösterme
- [x] AI ve jüri puanını karşılaştırma
- [ ] Düşük güvenli sonuç uyarısı
- [ ] Model sonucu ile proje sürümünü ilişkilendirme

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
