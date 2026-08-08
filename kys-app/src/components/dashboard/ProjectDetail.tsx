import React, { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { ShieldAlert, CheckCircle2, AlertTriangle, FileText, ChevronLeft, Bot, File, Award, Target, AlertCircle, ShieldX } from "lucide-react"
import { ProjectAiChat } from "./ProjectAiChat"

type ProjectDetail = {
  id: string;
  title: string;
  category: string;
  author: string;
  submit_date: string;
  status: string;
  score: {
    total: number;
    grade: string;
    category_fit: number;
    completeness: number;
    reference_quality: number;
    technical_depth: number;
  };
  similarity: {
    overall_score: number; // 0-1 (e.g. 0.15 for 15%)
    originality_label: string;
    matches: { title: string; source_type: string; similarity_score: number }[];
  };
  pdf_url?: string;
}

export function ProjectDetailView() {
  const [projectId, setProjectId] = useState<string | null>(null)
  const [data, setData] = useState<ProjectDetail | null>(null)
  const [loading, setLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<"pdf" | "ai">("ai")

  useEffect(() => {
    // URL'den ID'yi al
    if (typeof window !== 'undefined') {
      const urlParams = new URLSearchParams(window.location.search)
      const id = urlParams.get('id')
      if (id) {
        setProjectId(id)
        loadDetails(id)
      } else {
        setLoading(false) // ID yok
      }
    }
  }, [])

  async function loadDetails(id: string) {
    try {
      const detailData = await tauriInvoke("get_project_details", { id })
      setData(detailData)
    } catch (e) {
      console.error("Detay yüklenemedi:", e)
    } finally {
      setLoading(false)
    }
  }

  if (loading) {
    return <div className="p-8 text-center animate-pulse text-muted-foreground">Analiz verileri yükleniyor...</div>
  }

  if (!data) {
    return (
      <div className="p-8 text-center">
        <h2 className="text-xl font-bold mb-4">Proje Bulunamadı</h2>
        <a href="/dashboard" className="text-primary hover:underline">Dashboard'a Dön</a>
      </div>
    )
  }

  const isWarning = data.similarity.overall_score > 0.4;
  const isDanger = data.similarity.overall_score > 0.7;

  const getGradeIcon = (grade: string) => {
    if (grade.includes('A')) return <CheckCircle2 className="w-5 h-5 opacity-20" />;
    if (grade.includes('B')) return <Target className="w-5 h-5 opacity-20" />;
    if (grade.includes('C')) return <AlertTriangle className="w-5 h-5 opacity-20" />;
    return <ShieldAlert className="w-5 h-5 opacity-20" />;
  };

  const getGradeStyle = (grade: string) => {
    if (grade.includes('A')) return 'bg-emerald-50 border-emerald-200 text-emerald-700 dark:bg-emerald-950/30 dark:border-emerald-900/50 dark:text-emerald-400';
    if (grade.includes('B')) return 'bg-blue-50 border-blue-200 text-blue-700 dark:bg-blue-950/30 dark:border-blue-900/50 dark:text-blue-400';
    if (grade.includes('C')) return 'bg-amber-50 border-amber-200 text-amber-700 dark:bg-amber-950/30 dark:border-amber-900/50 dark:text-amber-400';
    return 'bg-rose-50 border-rose-200 text-rose-700 dark:bg-rose-950/30 dark:border-rose-900/50 dark:text-rose-400';
  };

  return (
    <div className="flex flex-col h-full">
      {/* Üst Bar / Geri Dön */}
      <div className="mb-6 flex items-center gap-4">
        <a href="/dashboard" className="p-2 bg-card border border-border rounded-lg hover:bg-muted transition-colors">
          <ChevronLeft className="w-5 h-5" />
        </a>
        <div>
          <h1 className="text-2xl font-bold">{data.title}</h1>
          <div className="flex items-center gap-3 text-sm text-muted-foreground mt-1">
            <span className="bg-primary/10 text-primary px-2 py-0.5 rounded text-xs font-semibold">{data.category}</span>
            <span>ID: {data.id}</span>
            <span>•</span>
            <span>Gönderen: <strong className="text-foreground">{data.author}</strong></span>
            <span>•</span>
            <span>Tarih: {data.submit_date}</span>
          </div>
        </div>
      </div>

      {/* İkiye Bölünmüş İçerik */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-6 min-h-0">
        
        {/* Sol Taraf: KYS Analiz Detayları */}
        <div className="flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2">
          
          {/* Ana Skor Kartı */}
          <div className="bg-card border border-border rounded-xl p-6 shadow-sm flex items-center justify-between">
            <div>
              <h3 className="text-lg font-semibold mb-1">Genel Değerlendirme Puanı</h3>
              <p className="text-sm text-muted-foreground">KYS-Engine yapay zeka ve kural tabanlı skorlaması.</p>
            </div>
            <div className="flex items-center gap-6">
              <div className="text-right">
                <div className="text-4xl font-black tracking-tighter">{data.score.total}<span className="text-xl font-medium text-muted-foreground">/100</span></div>
              </div>
              
              {/* Premium Grade Badge */}
              <div className={`w-16 h-16 rounded-[1.2rem] flex flex-col items-center justify-center border ${getGradeStyle(data.score.grade)} backdrop-blur-sm relative overflow-hidden group`}>
                <div className="absolute inset-0 bg-white/20 dark:bg-white/5 opacity-0 group-hover:opacity-100 transition-opacity"></div>
                <div className="flex items-center justify-center gap-1 z-10">
                  <span className="text-3xl font-black tracking-tighter leading-none mt-1">{data.score.grade}</span>
                </div>
                <div className="absolute -bottom-1 -right-1 opacity-20">
                  {getGradeIcon(data.score.grade)}
                </div>
              </div>

            </div>
          </div>

          {/* Metrikler */}
          <div className="bg-card border border-border rounded-xl p-6 shadow-sm">
            <h3 className="font-semibold mb-4">Değerlendirme Kırılımları</h3>
            <div className="space-y-5">
              <MetricBar label="Alan Uyumu" value={data.score.category_fit} color="bg-primary/80" />
              <MetricBar label="Bölüm Tamlığı" value={data.score.completeness} color="bg-primary/70" />
              <MetricBar label="Kaynak Kalitesi" value={data.score.reference_quality} color="bg-primary/60" />
              <MetricBar label="Teknik Derinlik" value={data.score.technical_depth} color="bg-primary/50" />
            </div>
          </div>

          {/* Benzerlik (İntihal) Raporu */}
          <div className={`bg-card border rounded-xl p-6 shadow-sm ${isDanger ? 'border-red-500/50' : isWarning ? 'border-yellow-500/50' : 'border-border'}`}>
            <div className="flex items-center justify-between mb-4">
              <div className="flex items-center gap-2">
                {isDanger ? <ShieldAlert className="w-5 h-5 text-red-500" /> : 
                 isWarning ? <AlertTriangle className="w-5 h-5 text-yellow-500" /> : 
                 <CheckCircle2 className="w-5 h-5 text-green-500" />}
                <h3 className="font-semibold">Özgünlük Analizi</h3>
              </div>
              <div className={`text-lg font-bold ${isDanger ? 'text-red-500' : isWarning ? 'text-yellow-500' : 'text-green-500'}`}>
                %{(data.similarity.overall_score * 100).toFixed(1)} Benzerlik
              </div>
            </div>
            
            <p className="text-sm text-muted-foreground mb-4">
              Sistem Kararı: <strong className="text-foreground">{data.similarity.originality_label}</strong>
            </p>

            {data.similarity.matches.length > 0 && (
              <div className="mt-4">
                <h4 className="text-xs font-bold uppercase text-muted-foreground mb-2">Bulunan Benzer Kaynaklar</h4>
                <ul className="space-y-2">
                  {data.similarity.matches.map((match, i) => (
                    <li key={i} className="text-sm flex items-center justify-between bg-muted/30 p-2 rounded border border-border">
                      <span className="truncate mr-4 flex-1">{match.title} <span className="text-muted-foreground text-xs">({match.source_type})</span></span>
                      <span className="font-mono text-red-400 font-medium">%{(match.similarity_score * 100).toFixed(1)}</span>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

        </div>

        {/* Sağ Taraf: Sekmeli Yapı (PDF / AI) */}
        <div className="bg-card border border-border rounded-xl shadow-sm flex flex-col overflow-hidden h-[700px] lg:h-auto">
          {/* Tab Headers */}
          <div className="h-14 border-b border-border flex items-center px-2 bg-muted/10 shrink-0">
            <button 
              onClick={() => setActiveTab('ai')}
              className={`flex-1 flex items-center justify-center gap-2 h-10 rounded-lg text-sm font-medium transition-colors ${
                activeTab === 'ai' 
                  ? 'bg-primary text-primary-foreground shadow' 
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              }`}
            >
              <Bot className="w-4 h-4" /> KYS AI Asistanı
            </button>
            <button 
              onClick={() => setActiveTab('pdf')}
              className={`flex-1 flex items-center justify-center gap-2 h-10 rounded-lg text-sm font-medium transition-colors ${
                activeTab === 'pdf' 
                  ? 'bg-primary text-primary-foreground shadow' 
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              }`}
            >
              <File className="w-4 h-4" /> Orijinal Rapor (PDF)
            </button>
          </div>
          
          {/* Tab Content */}
          <div className="flex-1 bg-zinc-900/50 relative overflow-hidden">
            {activeTab === 'ai' ? (
              <ProjectAiChat projectTitle={data.title} />
            ) : (
              data.pdf_url ? (
                <iframe 
                  src={data.pdf_url} 
                  className="w-full h-full border-0"
                  title="PDF Viewer"
                />
              ) : (
                <div className="absolute inset-0 flex flex-col items-center justify-center text-muted-foreground">
                  <FileText className="w-16 h-16 mb-4 opacity-20" />
                  <p>PDF dosyası bu ortamda yüklenemedi.</p>
                  <p className="text-xs mt-2">Geliştirme ortamında test ediliyor.</p>
                </div>
              )
            )}
          </div>
        </div>

      </div>
    </div>
  )
}

function MetricBar({ label, value, color }: { label: string, value: number, color: string }) {
  return (
    <div>
      <div className="flex justify-between mb-1 text-sm">
        <span className="font-medium text-muted-foreground">{label}</span>
        <span className="font-bold">{value}/100</span>
      </div>
      <div className="h-2 w-full bg-muted rounded-full overflow-hidden">
        <div className={`h-full ${color} rounded-full transition-all duration-1000 ease-out`} style={{ width: `${value}%` }} />
      </div>
    </div>
  )
}
