import React, { useEffect, useRef, useState } from "react"
import {
  Area, AreaChart, Bar, BarChart,
  ResponsiveContainer, Tooltip, XAxis, YAxis
} from "recharts"
import { tauriInvoke } from "@/lib/tauri"
import { FileText, BarChart2 } from "lucide-react"

type DailyWordPoint = { day: string; words: number; projects: number }
type DailyProjectPoint = { day: string; count: number }

type ChartData = {
  daily_words: DailyWordPoint[]
  daily_projects: DailyProjectPoint[]
}

// Recharts ResponsiveContainer sıfır-yükseklik sorununu çözmek için
function SafeChartWrapper({ children }: { children: React.ReactNode }) {
  const [isVisible, setIsVisible] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const timer = setTimeout(() => setIsVisible(true), 50)
    return () => clearTimeout(timer)
  }, [])

  return (
    <div ref={ref} className="h-64 w-full">
      {isVisible ? children : <div className="h-64 w-full bg-muted/10 rounded animate-pulse" />}
    </div>
  )
}

// Özel Tooltip: kelime sayısı için binlik ayraç
function WordTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-card border border-border rounded-lg px-3 py-2 shadow-md text-xs">
      <p className="font-semibold text-foreground mb-1">{label}</p>
      {payload.map((p: any) => (
        <div key={p.dataKey} className="flex items-center gap-2">
          <span className="w-2 h-2 rounded-full" style={{ backgroundColor: p.color }} />
          <span className="text-muted-foreground">
            {p.dataKey === "words" ? "Kelime" : "Proje"}:
          </span>
          <span className="font-bold text-foreground">
            {p.dataKey === "words"
              ? Number(p.value).toLocaleString("tr-TR")
              : p.value}
          </span>
        </div>
      ))}
    </div>
  )
}

function ProjectTooltip({ active, payload, label }: any) {
  if (!active || !payload?.length) return null
  return (
    <div className="bg-card border border-border rounded-lg px-3 py-2 shadow-md text-xs">
      <p className="font-semibold text-foreground mb-1">{label}</p>
      <div className="flex items-center gap-2">
        <span className="w-2 h-2 rounded-full bg-blue-400" />
        <span className="text-muted-foreground">Proje:</span>
        <span className="font-bold text-foreground">{payload[0]?.value}</span>
      </div>
    </div>
  )
}

export function DashboardCharts() {
  const [data, setData] = useState<ChartData>({ daily_words: [], daily_projects: [] })
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    async function loadData() {
      try {
        const res = await tauriInvoke("get_chart_data")
        if (res && res.daily_words) {
          setData(res as ChartData)
        }
      } catch (e) {
        console.error("Failed to load chart data:", e)
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  // Toplam kelime sayısı (tüm günlük verinin toplamı)
  const totalWords = data.daily_words.reduce((sum, d) => sum + (d.words || 0), 0)
  // Toplam proje sayısı (günlük toplamı)
  const totalDailyProjects = data.daily_projects.reduce((sum, d) => sum + (d.count || 0), 0)

  if (loading) {
    return (
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">
        {[1, 2].map(i => (
          <div key={i} className="bg-card rounded-xl p-6 border border-border h-80 animate-pulse">
            <div className="h-3 bg-muted rounded w-1/2 mb-3" />
            <div className="h-8 bg-muted rounded w-1/3 mb-8" />
            <div className="h-48 bg-muted/50 rounded" />
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-8">

      {/* ─── Sol: Haftalık Toplam Kelime Sayısı (Area Chart) ─── */}
      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col">
        <div className="mb-5">
          <div className="flex items-center gap-2 mb-1">
            <FileText className="w-4 h-4 text-primary opacity-70" />
            <h3 className="text-sm font-medium text-muted-foreground">
              Son 7 Günlük Analiz Edilen Kelime
            </h3>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-3xl font-bold">
              {totalWords > 0
                ? totalWords.toLocaleString("tr-TR")
                : "—"}
            </span>
            {totalWords >= 0 && (
              <span className="text-xs font-bold text-emerald-600 bg-emerald-500/10 px-2 py-1 rounded-full">
                Son 7 Gün
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Yüklenen PDF'lerdeki toplam kelime sayısı trendi
          </p>
        </div>
        <SafeChartWrapper>
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart
              data={data.daily_words}
              margin={{ top: 10, right: 0, left: -10, bottom: 0 }}
            >
              <defs>
                <linearGradient id="wordGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%"  stopColor="#3b82f6" stopOpacity={0.85} />
                  <stop offset="95%" stopColor="#3b82f6" stopOpacity={0.05} />
                </linearGradient>
                <linearGradient id="projectGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="5%"  stopColor="#60a5fa" stopOpacity={0.5} />
                  <stop offset="95%" stopColor="#60a5fa" stopOpacity={0} />
                </linearGradient>
              </defs>
              <XAxis
                dataKey="day"
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 11, fill: "hsl(var(--muted-foreground))" }}
                dy={8}
              />
              <YAxis
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 11, fill: "hsl(var(--muted-foreground))" }}
                tickFormatter={(v) =>
                  v >= 1000 ? `${(v / 1000).toFixed(0)}k` : `${v}`
                }
              />
              <Tooltip content={<WordTooltip />} />
              <Area
                type="monotone"
                dataKey="words"
                stroke="#3b82f6"
                strokeWidth={2}
                fill="url(#wordGradient)"
                dot={{ r: 3, fill: "#3b82f6", strokeWidth: 0 }}
                activeDot={{ r: 5, fill: "#3b82f6" }}
              />
            </AreaChart>
          </ResponsiveContainer>
        </SafeChartWrapper>
      </div>

      {/* ─── Sağ: Günlük İncelenen Proje Sayısı (Bar Chart) ─── */}
      <div className="bg-card rounded-xl p-6 shadow-sm border border-border flex flex-col">
        <div className="mb-5">
          <div className="flex items-center gap-2 mb-1">
            <BarChart2 className="w-4 h-4 text-primary opacity-70" />
            <h3 className="text-sm font-medium text-muted-foreground">
              Günlük Analiz Edilen Proje
            </h3>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-3xl font-bold">
              {totalDailyProjects > 0 ? totalDailyProjects : "—"}
            </span>
            {totalDailyProjects > 0 && (
              <span className="text-xs font-bold text-blue-600 bg-blue-500/10 px-2 py-1 rounded-full">
                Son 14 Gün
              </span>
            )}
          </div>
          <p className="text-xs text-muted-foreground mt-1">
            Gün bazında incelemeye giren proje sayısı
          </p>
        </div>
        <SafeChartWrapper>
          <ResponsiveContainer width="100%" height="100%">
            <BarChart
              data={data.daily_projects}
              margin={{ top: 10, right: 0, left: -10, bottom: 0 }}
              barSize={14}
            >
              <XAxis
                dataKey="day"
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 10, fill: "hsl(var(--muted-foreground))" }}
                dy={8}
                interval={1}
              />
              <YAxis
                axisLine={false}
                tickLine={false}
                tick={{ fontSize: 11, fill: "hsl(var(--muted-foreground))" }}
                allowDecimals={false}
              />
              <Tooltip content={<ProjectTooltip />} cursor={{ fill: "hsl(var(--muted) / 0.3)" }} />
              <Bar
                dataKey="count"
                fill="#3b82f6"
                radius={[4, 4, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </SafeChartWrapper>
      </div>

    </div>
  )
}
