import { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { ShieldAlert, CheckCircle2, AlertTriangle, FileText, ChevronLeft, File, Bot, Download } from "lucide-react"
import {
  Radar, RadarChart, PolarGrid, PolarAngleAxis, PolarRadiusAxis,
  ResponsiveContainer, Tooltip
} from "recharts"
import { ProjectCopilot } from "./ProjectCopilot"

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
    ai_probability?: number;
  };
  similarity: {
    overall_score: number; // 0-1 (e.g. 0.15 for 15%)
    originality_label: string;
    matches: { title: string; source_type: string; similarity_score: number; url?: string }[];
  };
  pdf_url?: string;
}

export function ProjectDetailView() {
  const [data, setData] = useState<ProjectDetail | null>(null)
  const [activeRightTab, setActiveRightTab] = useState<'pdf' | 'copilot'>('pdf')
  const [activeContext, setActiveContext] = useState<string>('')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (typeof window !== 'undefined') {
      const urlParams = new URLSearchParams(window.location.search)
      const id = urlParams.get('id')
      if (id) {
        loadDetails(id)
      } else {
        setLoading(false)
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


  const getGradeStyle = (grade: string) => {
    if (grade.includes('A')) return 'text-emerald-600 dark:text-emerald-500';
    if (grade.includes('B')) return 'text-blue-600 dark:text-blue-500';
    if (grade.includes('C')) return 'text-amber-600 dark:text-amber-500';
    return 'text-rose-600 dark:text-rose-500';
  };

  return (
    <div className="flex flex-col h-full print-full-width">
      {/* Üst Bar / Geri Dön */}
      <div className="mb-6 flex items-start sm:items-center justify-between gap-4 flex-col sm:flex-row">
        <div className="flex items-center gap-4">
          <a href="/dashboard" className="p-2 bg-card border border-border rounded-lg hover:bg-muted transition-colors no-print">
            <ChevronLeft className="w-5 h-5" />
          </a>
          <div>
            <h1 className="text-2xl font-bold">{data.title}</h1>
            <div className="flex flex-wrap items-center gap-3 text-sm text-muted-foreground mt-1">
              <span className="bg-primary/10 text-primary px-2 py-0.5 rounded text-xs font-semibold">{data.category}</span>
              <span>ID: {data.id}</span>
              <span className="hidden sm:inline">•</span>
              <span>Gönderen: <strong className="text-foreground">{data.author}</strong></span>
              <span className="hidden sm:inline">•</span>
              <span>Tarih: {data.submit_date}</span>
            </div>
          </div>
        </div>
        <button 
          onClick={() => window.print()}
          className="no-print flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors font-semibold text-sm shrink-0 shadow-sm"
        >
          <Download className="w-4 h-4" /> Raporu İndir (PDF)
        </button>
      </div>

      {/* İkiye Bölünmüş İçerik */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-2 gap-6 min-h-0">
        
        {/* Sol Taraf: JanissaryAsistan Analiz Detayları */}
        <div className="flex flex-col gap-6 overflow-y-auto custom-scrollbar pr-2">
          
          {/* Ana Skor Kartı */}
          <div className="bg-card border border-border rounded-xl p-6 shadow-sm flex items-center justify-between">
            <div>
              <h3 className="text-lg font-semibold mb-1">Genel Değerlendirme Puanı</h3>
              <p className="text-sm text-muted-foreground">JanissaryAsistan-Engine yapay zeka ve kural tabanlı skorlaması.</p>
            </div>
            <div className="flex items-center gap-6">
              <div className="text-right">
                <div className="text-4xl font-black tracking-tighter">{data.score.total}<span className="text-xl font-medium text-muted-foreground">/100</span></div>
              </div>
              
              {/* Premium Grade Badge (Kutusuz Sade Harf) */}
              <div className={`flex items-center justify-center ${getGradeStyle(data.score.grade)}`}>
                  <span className="text-5xl font-black tracking-tighter drop-shadow-sm">{data.score.grade}</span>
              </div>

            </div>
          </div>

          {/* Değerlendirme Kırılımları */}
          <div className="bg-card border border-border rounded-xl p-6 shadow-sm">
            <h3 className="font-semibold mb-4">Değerlendirme Kırılımları</h3>
            <div className="space-y-5">
              <MetricBar label="Alan Uyumu" value={data.score.category_fit} color="bg-primary/80" />
              <MetricBar label="Bölüm Tamlığı" value={data.score.completeness} color="bg-primary/70" />
              <MetricBar label="Kaynak Kalitesi" value={data.score.reference_quality} color="bg-primary/60" />
              <MetricBar label="Teknik Derinlik" value={data.score.technical_depth} color="bg-primary/50" />
              {data.score.ai_probability !== undefined && (
                <div className="mt-6 pt-5 border-t border-border">
                  <MetricBar 
                    label="Yapay Zeka İçeriği (İhtimali)" 
                    value={Math.round(data.score.ai_probability)} 
                    color={data.score.ai_probability > 60 ? "bg-red-500" : "bg-yellow-500/80"} 
                  />
                  {data.score.ai_probability > 60 && (
                     <p className="text-xs text-red-500 mt-2 font-medium flex items-center gap-1.5"><AlertTriangle className="w-3.5 h-3.5"/> Yüksek oranda yapay zeka içeriği tespit edildi. Puan kırıldı!</p>
                  )}
                </div>
              )}
            </div>
          </div>

          {/* ─── Radar Grafik: Nihai Puan Analizi ─── */}
          <div className="bg-card border border-border rounded-xl p-6 shadow-sm">
            <div className="mb-4">
              <h3 className="font-semibold">Puan Radar Analizi</h3>
              <p className="text-xs text-muted-foreground mt-0.5">Tüm değerlendirme kriterlerinin görsel karşılaştırması</p>
            </div>
            <ScoreRadarChart score={data.score} />
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
                <h4 className="text-xs font-bold uppercase text-muted-foreground mb-2 flex items-center gap-2">
                  <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
                  İnternet Araştırması & Kaynak Kanıtları
                </h4>
                <ul className="space-y-2">
                  {data.similarity.matches.map((match, i) => (
                    <li key={i} className="text-sm flex items-center justify-between bg-muted/30 p-2 rounded border border-border group">
                      <span className="truncate mr-4 flex-1">
                        {match.url ? (
                          <a href={match.url} target="_blank" rel="noreferrer" className="hover:underline text-primary font-medium">
                            {match.title}
                          </a>
                        ) : (
                          match.title
                        )}{" "}
                        <span className="text-muted-foreground text-xs">({match.source_type})</span>
                      </span>
                      <div className="flex items-center gap-3">
                        <span className="font-mono text-red-400 font-medium">%{(match.similarity_score * 100).toFixed(1)}</span>
                        <button 
                          onClick={() => {
                            setActiveContext(match.title);
                            setActiveRightTab('copilot');
                          }}
                          className="opacity-0 group-hover:opacity-100 p-1.5 hover:bg-primary/20 text-primary rounded transition-all"
                          title="PDF'deki Kelime Analizi İçin Copilot'a Sor"
                        >
                          <Bot className="w-4 h-4" />
                        </button>
                      </div>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </div>

        </div>

        {/* Sağ Taraf: Orijinal Rapor (PDF) */}
        <div className="bg-card border border-border rounded-xl shadow-sm flex flex-col overflow-hidden h-[700px] lg:h-auto">
          {/* Header (Tabs) */}
          <div className="h-14 border-b border-border flex bg-muted/10 shrink-0">
            <button 
              onClick={() => setActiveRightTab('pdf')}
              className={`flex-1 flex items-center justify-center gap-2 font-semibold text-sm transition-colors border-b-2 ${activeRightTab === 'pdf' ? 'border-primary text-primary bg-background/50' : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-muted/30'}`}
            >
              <File className="w-4 h-4" /> Orijinal Rapor
            </button>
            <button 
              onClick={() => setActiveRightTab('copilot')}
              className={`flex-1 flex items-center justify-center gap-2 font-semibold text-sm transition-colors border-b-2 ${activeRightTab === 'copilot' ? 'border-primary text-primary bg-background/50' : 'border-transparent text-muted-foreground hover:text-foreground hover:bg-muted/30'}`}
            >
              <Bot className="w-4 h-4" /> Janissary Copilot
            </button>
          </div>
          
          {/* Content */}
          <div className="flex-1 bg-zinc-900/50 relative overflow-hidden">
            {activeRightTab === 'pdf' ? (
              data.pdf_url ? (
                <PdfViewer url={data.pdf_url} />
              ) : (
                <div className="absolute inset-0 flex flex-col items-center justify-center text-muted-foreground">
                  <FileText className="w-16 h-16 mb-4 opacity-20" />
                  <p>PDF dosyası bulunamadı.</p>
                  <p className="text-xs mt-2">Proje doğru şekilde yüklendi mi kontrol edin.</p>
                </div>
              )
            ) : (
              <ProjectCopilot projectId={data.id} projectTitle={data.title} initialContext={activeContext} />
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

// ─── Radar Grafik Bileşeni ─────────────────────────────────────────

type ScoreData = {
  total: number
  grade: string
  category_fit: number
  completeness: number
  reference_quality: number
  technical_depth: number
  ai_probability?: number
}

function RadarTooltipContent({ active, payload }: any) {
  if (!active || !payload?.length) return null
  const d = payload[0]?.payload
  if (!d) return null
  return (
    <div className="bg-card border border-border rounded-lg px-3 py-2 shadow-md text-xs">
      <p className="font-semibold text-foreground mb-1">{d.kriter}</p>
      <p className="text-muted-foreground">
        Puan: <span className="font-bold text-foreground">{d.puan}/100</span>
      </p>
    </div>
  )
}

function ScoreRadarChart({ score }: { score: ScoreData }) {
  const radarData = [
    { kriter: "Alan Uyumu",      puan: score.category_fit },
    { kriter: "Bölüm Tamlığı",  puan: score.completeness },
    { kriter: "Kaynak Kalitesi", puan: score.reference_quality },
    { kriter: "Teknik Derinlik", puan: score.technical_depth },
    ...(score.ai_probability !== undefined ? [{ kriter: "İnsan Katkısı", puan: Math.round(100 - score.ai_probability) }] : []),
    { kriter: "Genel Puan",      puan: score.total },
  ]

  return (
    <div className="w-full h-64">
      <ResponsiveContainer width="100%" height="100%">
        <RadarChart data={radarData} margin={{ top: 10, right: 30, left: 30, bottom: 10 }}>
          <defs>
            <linearGradient id="radarFill" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%"   stopColor="#3b82f6" stopOpacity={0.6} />
              <stop offset="100%" stopColor="#3b82f6" stopOpacity={0.1} />
            </linearGradient>
          </defs>
          <PolarGrid
            stroke="hsl(var(--border))"
            strokeDasharray="3 3"
          />
          <PolarAngleAxis
            dataKey="kriter"
            tick={{
              fontSize: 11,
              fill: "hsl(var(--muted-foreground))",
              fontWeight: 500,
            }}
          />
          <PolarRadiusAxis domain={[0, 100]} tick={false} axisLine={false} />
          <Tooltip content={<RadarTooltipContent />} />
          <Radar
            dataKey="puan"
            stroke="#3b82f6"
            strokeWidth={2}
            fill="url(#radarFill)"
            dot={{ r: 4, fill: "#3b82f6", strokeWidth: 0 }}
            activeDot={{ r: 6, fill: "#3b82f6", stroke: "#93c5fd", strokeWidth: 2 }}
          />
        </RadarChart>
      </ResponsiveContainer>
    </div>
  )
}

// ─── PDF Viewer Bileşeni ──────────────────────────────────────────
// Blob URL kullanır — tarayıcı güvenlik engeli olmadan doğrudan gösterir

function PdfViewer({ url }: { url: string }) {
  const [blobUrl, setBlobUrl] = useState<string | null>(null)
  const [pdfError, setPdfError] = useState<string | null>(null)
  const [pdfLoading, setPdfLoading] = useState(true)

  useEffect(() => {
    let objectUrl: string | null = null

    fetch(url)
      .then(res => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.blob()
      })
      .then(blob => {
        objectUrl = URL.createObjectURL(blob)
        setBlobUrl(objectUrl)
        setPdfLoading(false)
      })
      .catch(err => {
        setPdfError(err.message)
        setPdfLoading(false)
      })

    return () => {
      if (objectUrl) URL.revokeObjectURL(objectUrl)
    }
  }, [url])

  if (pdfLoading) {
    return (
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-muted-foreground">
        <div className="w-10 h-10 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
        <p className="text-sm">PDF yükleniyor...</p>
      </div>
    )
  }

  if (pdfError || !blobUrl) {
    return (
      <div className="absolute inset-0 flex flex-col items-center justify-center gap-3 text-muted-foreground">
        <FileText className="w-16 h-16 opacity-20" />
        <p className="text-sm">PDF görüntülenemedi.</p>
        <p className="text-xs text-red-400">{pdfError}</p>
        <a
          href={url}
          target="_blank"
          rel="noreferrer"
          className="mt-2 px-4 py-2 bg-primary text-white rounded-lg text-sm font-semibold hover:bg-primary/90 transition-colors"
        >
          Yeni sekmede aç →
        </a>
      </div>
    )
  }

  return (
    <div className="w-full h-full flex flex-col">
      <div className="h-10 bg-zinc-800/80 border-b border-white/5 flex items-center justify-between px-3 shrink-0">
        <span className="text-xs text-zinc-400 font-medium">📄 PDF Görüntüleyici</span>
        <a
          href={blobUrl}
          download="proje.pdf"
          className="text-xs text-primary hover:underline font-medium"
        >
          ⬇ İndir
        </a>
      </div>
      <object
        data={blobUrl}
        type="application/pdf"
        className="flex-1 w-full"
        style={{ minHeight: 0 }}
      >
        <iframe
          src={blobUrl}
          className="w-full h-full border-0"
          title="PDF Görüntüleyici"
        />
      </object>
    </div>
  )
}
