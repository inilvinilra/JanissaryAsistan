import { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"

type Stats = {
  total_projects: string;
  total_projects_trend: string;
  avg_score: string;
  avg_score_trend: string;
  risk_projects: string;
  risk_projects_trend: string;
}

// Hata durumunda gösterilecek fallback istatistikler
const FALLBACK_STATS: Stats = {
  total_projects: "—",
  total_projects_trend: "+0%",
  avg_score: "—",
  avg_score_trend: "+0%",
  risk_projects: "—",
  risk_projects_trend: "+0%",
}

export function DashboardOverview() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function loadData() {
      try {
        const data = await tauriInvoke("get_dashboard_stats")
        if (data && typeof data === "object") {
          setStats(data as Stats)
        } else {
          setStats(FALLBACK_STATS)
        }
      } catch (e) {
        console.error("Dashboard stats hatası:", e)
        setStats(FALLBACK_STATS)
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8 animate-pulse">
        {[1, 2, 3].map(i => (
          <div key={i} className="bg-card rounded-xl p-6 shadow-sm border border-border h-32">
            <div className="h-3 bg-muted rounded w-2/3 mb-4" />
            <div className="h-8 bg-muted rounded w-1/2" />
          </div>
        ))}
      </div>
    )
  }

  // stats her zaman dolu olacak (null durumda fallback atıyoruz)
  const s = stats!

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
      <StatCard
        label="Analiz Edilen Projeler"
        value={s.total_projects}
        trend={s.total_projects_trend}
        subtitle="Son 30 gün"
      />
      <StatCard
        label="Ortalama Başarı Puanı"
        value={s.avg_score}
        trend={s.avg_score_trend}
        subtitle="Son 30 gün"
      />
      <StatCard
        label="Riskli / Kopya Projeler"
        value={s.risk_projects}
        trend={s.risk_projects_trend}
        subtitle="Son 30 gün"
        trendInverse // Bu kart için +% kötü, -% iyidir
      />
    </div>
  )
}

function StatCard({
  label, value, trend, subtitle, trendInverse = false
}: {
  label: string;
  value: string | number;
  trend?: string;
  subtitle: string;
  trendInverse?: boolean;
}) {
  const safeTrend = trend || "+0%"
  const isPositive = safeTrend.startsWith('+')
  // trendInverse=true ise pozitif trend kötüdür (riskli projeler arttı = kötü)
  const isGood = trendInverse ? !isPositive : isPositive

  return (
    <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col gap-2">
      <p className="text-sm font-medium text-muted-foreground">{label}</p>
      <div className="flex items-center justify-between">
        <p className="text-4xl font-bold">{value}</p>
        <span className={`text-xs font-bold px-2 py-1 rounded-full ${
          isGood
            ? 'text-green-600 bg-green-500/10'
            : 'text-red-600 bg-red-500/10'
        }`}>
          {safeTrend}
        </span>
      </div>
      <p className="text-xs text-muted-foreground mt-2">{subtitle}</p>
    </div>
  )
}
