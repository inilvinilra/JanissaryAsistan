export async function tauriInvoke(cmd: string, args?: any): Promise<any> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke(cmd, args);
  }
  // Tarayıcıda test modu (mock veriler)
  console.log(`[Tauri Mock] Called command: ${cmd}`, args);
  
  if (cmd === "get_recent_projects") {
    return [
      { id: "PRJ-2041", title: "Görüntü İşleme ile Yüz Tanıma", category: "Yapay Zeka", score: 92, grade: "A", status: "Tamamlandı" },
      { id: "PRJ-2042", title: "Otonom Tarım Robotu", category: "Robotik", score: 85, grade: "B+", status: "Tamamlandı" },
      { id: "PRJ-2043", title: "Akıllı Ev Güvenlik Sistemi", category: "Nesnelerin İnterneti", score: 72, grade: "C", status: "Uyarı: Benzerlik" },
      { id: "PRJ-2044", title: "Güneş Paneli Verimlilik Analizi", category: "Enerji", score: 45, grade: "F", status: "Kopya İhtimali" },
      { id: "PRJ-2045", title: "Deprem Erken Uyarı Ağı", category: "Afet Yönetimi", score: null, grade: "-", status: "İnceleniyor" },
    ];
  }
  
  if (cmd === "get_dashboard_stats") {
    return {
      total_projects: "14k",
      total_projects_trend: "+25%",
      avg_score: "325",
      avg_score_trend: "-25%",
      risk_projects: "200k",
      risk_projects_trend: "+5%"
    };
  }

  if (cmd === "get_project_details") {
    const id = args?.id || "PRJ-2041";
    
    // Basit bir mock veritabanı simülasyonu
    const mockDB: Record<string, any> = {
      "PRJ-2041": { title: "Görüntü İşleme ile Yüz Tanıma", category: "Yapay Zeka", score: 92, grade: "A", sim: 0.12, label: "Özgünlük Yüksek (Özgün)" },
      "PRJ-2042": { title: "Otonom Tarım Robotu", category: "Robotik", score: 85, grade: "B", sim: 0.25, label: "Özgünlük Kabul Edilebilir" },
      "PRJ-2043": { title: "Akıllı Ev Güvenlik Sistemi", category: "Nesnelerin İnterneti", score: 72, grade: "C", sim: 0.45, label: "Uyarı: Yüksek Benzerlik" },
      "PRJ-2044": { title: "Güneş Paneli Verimlilik Analizi", category: "Enerji", score: 45, grade: "F", sim: 0.75, label: "Kopya/Çok Yüksek Benzerlik" },
    };

    // ID'nin sonundaki tireli varyasyonları (Örn: PRJ-2041-1) ana formata çevir
    const baseId = id.split("-").slice(0, 2).join("-");
    const project = mockDB[baseId] || mockDB["PRJ-2041"];

    return {
      id: id,
      title: project.title,
      category: project.category,
      author: "Sistem Kullanıcısı (Takım)",
      submit_date: "14 Mayıs 2026",
      status: project.score >= 50 ? "Tamamlandı" : "Kritik",
      score: {
        total: project.score,
        grade: project.grade,
        category_fit: Math.min(100, project.score + 3),
        completeness: Math.max(0, project.score - 4),
        reference_quality: Math.min(100, project.score + 2),
        technical_depth: Math.max(0, project.score - 1)
      },
      similarity: {
        overall_score: project.sim,
        originality_label: project.label,
        matches: project.sim > 0.4 ? [
          { title: "İnternette Bulunan Benzer Bir Çalışma", source_type: "Akademik Makale", similarity_score: project.sim / 2 },
          { title: "Geçen Yılın Projesi", source_type: "Arşiv", similarity_score: project.sim / 3 }
        ] : [
          { title: "Genel Konsept Benzerliği", source_type: "Blog", similarity_score: 0.04 }
        ]
      },
      pdf_url: undefined
    };
  }

  if (cmd === "analyze_project_pdf") {
    return new Promise(resolve => {
      setTimeout(() => {
        resolve({
          id: "PRJ-9999",
          title: "Test Yüklemesi (Tarayıcı Simülasyonu)",
          category: "Veri Bilimi",
          author: "Test Kullanıcı",
          submit_date: "Şimdi",
          status: "Yeni Analiz",
          score: {
            total: 88,
            grade: "B+",
            category_fit: 90,
            completeness: 85,
            reference_quality: 80,
            technical_depth: 90
          },
          similarity: {
            overall_score: 0.05,
            originality_label: "Çok Özgün",
            matches: []
          },
          pdf_url: undefined
        });
      }, 2000); // 2 saniye yükleme animasyonu testi
    });
  }

  return { success: true, message: "Browser test mode" };
}
