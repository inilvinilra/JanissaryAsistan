-- =====================================================
-- JanissaryAsistan (Karar Yönetim Sistemi) — Supabase DB Migrasyonu
-- Supabase Dashboard → SQL Editor'a yapıştır ve çalıştır
-- =====================================================

-- 1. Projeler tablosu
CREATE TABLE IF NOT EXISTS projects (
    id          SERIAL PRIMARY KEY,
    filename    TEXT NOT NULL,
    file_type   TEXT NOT NULL DEFAULT 'Pdf',
    word_count  INTEGER DEFAULT 0,
    language    TEXT DEFAULT 'Turkish',
    category    TEXT DEFAULT 'Genel',
    grade       TEXT DEFAULT '-',
    status      TEXT DEFAULT 'İnceleniyor',
    created_at  TIMESTAMPTZ DEFAULT NOW()
);

-- 2. Puanlama tablosu
CREATE TABLE IF NOT EXISTS scores (
    id                  SERIAL PRIMARY KEY,
    project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    category_fit        REAL DEFAULT 0,
    completeness        REAL DEFAULT 0,
    reference_quality   REAL DEFAULT 0,
    technical_depth     REAL DEFAULT 0,
    originality         REAL DEFAULT 0,
    total_score         REAL DEFAULT 0,
    grade               TEXT DEFAULT '-',
    reason              TEXT,
    created_at          TIMESTAMPTZ DEFAULT NOW()
);

-- 3. Benzerlik eşleşmeleri tablosu
CREATE TABLE IF NOT EXISTS similarity_matches (
    id                  SERIAL PRIMARY KEY,
    project_id          INTEGER REFERENCES projects(id) ON DELETE CASCADE,
    title               TEXT,
    url                 TEXT,
    source_type         TEXT DEFAULT 'Web',
    similarity_score    REAL DEFAULT 0,
    matched_keywords    TEXT,
    explanation         TEXT
);

-- 4. Row Level Security (RLS) — Authenticated kullanıcılar her şeyi yapabilsin
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE scores ENABLE ROW LEVEL SECURITY;
ALTER TABLE similarity_matches ENABLE ROW LEVEL SECURITY;

-- Projects politikaları
DROP POLICY IF EXISTS "kys_projects_all" ON projects;
CREATE POLICY "kys_projects_all" ON projects FOR ALL USING (true) WITH CHECK (true);

-- Scores politikaları
DROP POLICY IF EXISTS "kys_scores_all" ON scores;
CREATE POLICY "kys_scores_all" ON scores FOR ALL USING (true) WITH CHECK (true);

-- Similarity politikaları
DROP POLICY IF EXISTS "kys_sim_all" ON similarity_matches;
CREATE POLICY "kys_sim_all" ON similarity_matches FOR ALL USING (true) WITH CHECK (true);

-- 5. Demo veriler (test için)
INSERT INTO projects (filename, file_type, word_count, language, category, grade, status) VALUES
    ('Görüntü İşleme ile Yüz Tanıma.pdf', 'Pdf', 4500, 'Turkish', 'Yapay Zeka', 'A', 'Tamamlandı'),
    ('Otonom Tarım Robotu.pdf', 'Pdf', 3800, 'Turkish', 'Robotik', 'B+', 'Tamamlandı'),
    ('Akıllı Ev Güvenlik Sistemi.pdf', 'Pdf', 2900, 'Turkish', 'Nesnelerin İnterneti', 'C', 'Uyarı: Benzerlik'),
    ('Güneş Paneli Verimlilik Analizi.pdf', 'Pdf', 1800, 'Turkish', 'Enerji', 'F', 'Kopya İhtimali'),
    ('Deprem Erken Uyarı Ağı.pdf', 'Pdf', 5200, 'Turkish', 'Afet Yönetimi', '-', 'İnceleniyor')
ON CONFLICT DO NOTHING;

-- 6. Demo puanlar
INSERT INTO scores (project_id, category_fit, completeness, reference_quality, technical_depth, originality, total_score, grade)
SELECT p.id, 95, 90, 88, 92, 95, 92, 'A'
FROM projects p WHERE p.filename = 'Görüntü İşleme ile Yüz Tanıma.pdf';

INSERT INTO scores (project_id, category_fit, completeness, reference_quality, technical_depth, originality, total_score, grade)
SELECT p.id, 88, 82, 85, 84, 79, 85, 'B+'
FROM projects p WHERE p.filename = 'Otonom Tarım Robotu.pdf';

INSERT INTO scores (project_id, category_fit, completeness, reference_quality, technical_depth, originality, total_score, grade)
SELECT p.id, 74, 70, 68, 72, 55, 72, 'C'
FROM projects p WHERE p.filename = 'Akıllı Ev Güvenlik Sistemi.pdf';

INSERT INTO scores (project_id, category_fit, completeness, reference_quality, technical_depth, originality, total_score, grade)
SELECT p.id, 50, 42, 38, 44, 25, 45, 'F'
FROM projects p WHERE p.filename = 'Güneş Paneli Verimlilik Analizi.pdf';

-- 7. Demo benzerlik eşleşmeleri
INSERT INTO similarity_matches (project_id, title, source_type, similarity_score)
SELECT p.id, 'ResNet Tabanlı Yüz Tanıma Sistemi (IEEE 2023)', 'Akademik Makale', 0.12
FROM projects p WHERE p.filename = 'Akıllı Ev Güvenlik Sistemi.pdf';

INSERT INTO similarity_matches (project_id, title, source_type, similarity_score)
SELECT p.id, '2022 Yılı Bitirme Projesi Arşivi', 'Arşiv', 0.38
FROM projects p WHERE p.filename = 'Akıllı Ev Güvenlik Sistemi.pdf';

INSERT INTO similarity_matches (project_id, title, source_type, similarity_score)
SELECT p.id, 'Github: solar-panel-efficiency', 'GitHub Repo', 0.71
FROM projects p WHERE p.filename = 'Güneş Paneli Verimlilik Analizi.pdf';

INSERT INTO similarity_matches (project_id, title, source_type, similarity_score)
SELECT p.id, 'T3 2024 Final Projesi - Enerji', 'Arşiv', 0.52
FROM projects p WHERE p.filename = 'Güneş Paneli Verimlilik Analizi.pdf';

-- Hazır! Tüm tablolar ve demo veriler oluşturuldu.
SELECT 'Kurulum tamamlandı!' as durum, 
       (SELECT COUNT(*) FROM projects) as proje_sayisi,
       (SELECT COUNT(*) FROM scores) as puan_sayisi;
