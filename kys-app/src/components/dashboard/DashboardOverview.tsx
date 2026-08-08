import React, { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"

type Stats = {
  total_projects: string;
  total_projects_trend: string;
  avg_score: string;
  avg_score_trend: string;
  risk_projects: string;
  risk_projects_trend: string;
}

export function DashboardOverview() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function loadData() {
      try {
        const data = await tauriInvoke("get_dashboard_stats")
        setStats(data)
      } catch (e) {
        console.error("Failed to load stats", e)
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  if (loading || !stats) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8 animate-pulse">
        {[1, 2, 3].map(i => (
          <div key={i} className="bg-card rounded-xl p-6 shadow-sm border border-border h-32"></div>
        ))}
      </div>
    )
  }

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col gap-2">
        <p className="text-sm font-medium text-muted-foreground">Analiz Edilen Projeler</p>
        <div className="flex items-center justify-between">
          <p className="text-4xl font-bold">{stats.total_projects}</p>
          <span className={`text-xs font-bold px-2 py-1 rounded-full ${
            stats.total_projects_trend.startsWith('+') ? 'text-green-600 bg-green-500/10' : 'text-red-600 bg-red-500/10'
          }`}>
            {stats.total_projects_trend}
          </span>
        </div>
        <p className="text-xs text-muted-foreground mt-2">Son 30 gün</p>
      </div>

      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col gap-2">
        <p className="text-sm font-medium text-muted-foreground">Ortalama Başarı Puanı</p>
        <div className="flex items-center justify-between">
          <p className="text-4xl font-bold">{stats.avg_score}</p>
          <span className={`text-xs font-bold px-2 py-1 rounded-full ${
            stats.avg_score_trend.startsWith('+') ? 'text-green-600 bg-green-500/10' : 'text-red-600 bg-red-500/10'
          }`}>
            {stats.avg_score_trend}
          </span>
        </div>
        <p className="text-xs text-muted-foreground mt-2">Son 30 gün</p>
      </div>

      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col gap-2">
        <p className="text-sm font-medium text-muted-foreground">Riskli/Kopya Projeler</p>
        <div className="flex items-center justify-between">
          <p className="text-4xl font-bold">{stats.risk_projects}</p>
          <span className={`text-xs font-bold px-2 py-1 rounded-full ${
            stats.risk_projects_trend.startsWith('+') ? 'text-green-600 bg-green-500/10' : 'text-red-600 bg-red-500/10'
          }`}>
            {stats.risk_projects_trend}
          </span>
        </div>
        <p className="text-xs text-muted-foreground mt-2">Son 30 gün</p>
      </div>
    </div>
  )
}
