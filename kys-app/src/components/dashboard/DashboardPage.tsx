import React, { useState } from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import { DashboardOverview } from "@/components/dashboard/DashboardOverview"
import { DashboardCharts } from "@/components/dashboard/DashboardCharts"
import { RecentProjectsTable } from "@/components/dashboard/RecentProjectsTable"

/**
 * DashboardPage - Tüm dashboard içeriğini saran tek React bileşeni.
 * Astro'da `client:only="react"` direktifi ile kullanılmalı.
 * AuthGuard + Sidebar + Header + İçerik tek bir bileşen ağacında.
 */
export function DashboardPage() {
  const [currentCategory, setCurrentCategory] = useState<string | null>(null)

  React.useEffect(() => {
    if (typeof window !== "undefined") {
      const params = new URLSearchParams(window.location.search)
      setCurrentCategory(params.get("category"))
    }
  }, [])

  return (
    <AuthGuard>
      <div className="flex h-screen w-full bg-background text-foreground font-sans antialiased overflow-hidden">
        {/* Sol Menü */}
        <Sidebar />

        {/* Sağ İçerik Alanı */}
        <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
          {/* Üst Header */}
          <header className="h-20 border-b border-border bg-card/50 flex items-center px-6 justify-between shrink-0">
            <div className="font-semibold text-lg text-muted-foreground flex items-center gap-2">
              <span className="text-foreground">JanissaryAsistan</span> / Dashboard
            </div>
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>

          {/* Kaydırılabilir Ana Alan */}
          <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar bg-background">
            <div className="max-w-7xl mx-auto">
              <div className="mb-8 flex items-center justify-between">
                <div>
                  <h1 className="text-3xl font-bold tracking-tight mb-2">
                    {currentCategory ? `Kategori: ${currentCategory}` : "Genel Bakış"}
                  </h1>
                  <p className="text-muted-foreground">
                    {currentCategory ? "Bu kategorideki projeler ve manuel sıralama görünümü." : "Tüm analiz metrikleri ve değerlendirme özetleri."}
                  </p>
                </div>
                <button
                  onClick={() => window.location.href = "/dashboard"}
                  className="px-4 py-2 bg-muted/50 hover:bg-muted text-foreground text-sm font-medium rounded-lg border border-border transition-colors flex items-center gap-2"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
                  Sıfırla
                </button>
              </div>

              {!currentCategory && (
                <>
                  <DashboardOverview />
                  <DashboardCharts />
                </>
              )}
              <RecentProjectsTable />
            </div>
          </main>
        </div>
      </div>
    </AuthGuard>
  )
}
