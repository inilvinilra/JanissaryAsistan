use super::*;

fn assert_language(text: &str, expected: &str) {
    let detected = detect(text);
    assert_eq!(
        detected.name(),
        expected,
        "\n  metin: {}\n  beklenen: {expected}, bulunan: {detected}",
        text.chars().take(60).collect::<String>()
    );
}

#[test]
fn detects_turkish() {
    assert_language(
        "bu proje için çalışma yapıldı ancak veya olan durumlar önemli",
        "Turkish",
    );
}

#[test]
fn detects_turkish_prose_without_the_listed_stopwords() {
    assert_language(
        "Geliştirilen sistemin ayrıntıları anlatılmaktadır. Önerilen mimari, \
         veri toplama, ön işleme, model eğitimi ve değerlendirme aşamalarından \
         oluşmaktadır. Bulgular literatürdeki sonuçlarla karşılaştırılmıştır.",
        "Turkish",
    );
}

/// The most common real submission shape: Turkish prose carrying English
/// technical vocabulary, which trips purely statistical detectors.
#[test]
fn detects_turkish_carrying_english_technical_terms() {
    assert_language(
        "Sistemimiz bir REST API üzerinden çalışır. Backend tarafında PostgreSQL \
         veritabanı, frontend tarafında React kullanılmıştır. Deployment sürecinde \
         Docker container yapısı tercih edilmiş, CI/CD pipeline kurulmuştur.",
        "Turkish",
    );
}

#[test]
fn detects_english_technical_prose() {
    assert_language(
        "The proposed architecture consists of data collection, preprocessing, \
         model training and evaluation stages. Results are compared with the \
         findings reported in the literature.",
        "English",
    );
}

#[test]
fn detects_western_european_languages() {
    assert_language(
        "Die vorgeschlagene Architektur besteht aus mehreren Schichten, die \
         gemeinsam die Datenverarbeitung übernehmen und die Ergebnisse anschließend \
         auswerten und darstellen.",
        "German",
    );
    assert_language(
        "L'architecture proposée comprend plusieurs couches qui assurent ensemble \
         le traitement des données puis évaluent et présentent les résultats obtenus.",
        "French",
    );
    assert_language(
        "La arquitectura propuesta consta de varias capas que juntas realizan el \
         procesamiento de los datos y luego evalúan y presentan los resultados.",
        "Spanish",
    );
}

/// Languages the previous 70-language detector could not name at all.
#[test]
fn detects_supported_languages_beyond_turkish_and_english() {
    assert_language(
        "معماری پیشنهادی از چندین لایه تشکیل شده است که با هم پردازش داده ها را \
         انجام می دهند و سپس نتایج را ارزیابی و ارائه می کنند",
        "Persian",
    );
}

#[test]
fn unsupported_languages_remain_unknown() {
    assert_language(
        "Usanifu uliopendekezwa una tabaka kadhaa ambazo kwa pamoja hufanya \
         uchakataji wa data na kisha kutathmini na kuwasilisha matokeo yaliyopatikana",
        "Unknown",
    );
}

/// German and French share ç/ö/ü with Turkish, so the orthographic override
/// must not claim them.
#[test]
fn shared_diacritics_do_not_force_a_turkish_verdict() {
    assert_language(
        "Die Übertragung der Messwerte erfolgt über eine gesicherte Verbindung, \
         während die Auswertung größtenteils auf dem Server stattfindet und die \
         Ergebnisse später zusammengeführt werden.",
        "German",
    );
}

/// Azerbaijani uses the same ğ/ı/ş as Turkish; only the schwa separates them.
#[test]
fn azerbaijani_is_not_reported_as_turkish() {
    assert_language(
        "Təklif olunan arxitektura bir neçə təbəqədən ibarətdir və bu təbəqələr \
         birlikdə məlumatların emalını həyata keçirir. Ölçmələr təhlükəsiz bağlantı \
         vasitəsilə ötürülür və sonra serverdə qiymətləndirilir.",
        "Azerbaijani",
    );
}

/// The two competition languages must resolve even in a single sentence, where
/// the statistical model declines to commit.
#[test]
fn short_reports_in_the_competition_languages_still_resolve() {
    assert_language(
        "Our project aims to reduce water consumption in agricultural irrigation.",
        "English",
    );
    assert_language(
        "Projemiz tarımsal sulamada su tüketimini azaltmayı hedefliyor.",
        "Turkish",
    );
}

#[test]
fn text_too_short_or_symbolic_stays_unknown() {
    assert_language("", "Unknown");
    assert_language("--- 123 456 --- [1] [2] 2026", "Unknown");
    assert_language("Kısa.", "Unknown");
}

#[test]
fn the_supported_list_is_sorted_and_free_of_duplicates() {
    let names = supported_names();
    assert!(names.len() >= 60, "beklenenden az dil: {}", names.len());
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(names, sorted);
    assert!(names.contains(&"Turkish".to_string()));
    assert!(names.contains(&"Persian".to_string()));
}
