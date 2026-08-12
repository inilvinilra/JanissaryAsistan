import React from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import { UploadsTable } from "@/components/dashboard/UploadsTable"
import { useEffect, useState } from "react"

export function UploadsPage() {
  const [categoryName, setCategoryName] = useState<string | null>(null)

  useEffect(() => {
    if (typeof window !== "undefined") {
      const urlParams = new URLSearchParams(window.location.search)
      const name = urlParams.get('name')
      if (name) setCategoryName(name)
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
              <span className="text-foreground">JanissaryAsistan</span> / Yüklenenler
            </div>
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>

          {/* Kaydırılabilir Ana Alan */}
          <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar bg-background">
            <div className="max-w-[1200px] mx-auto">
              <div className="mb-8 flex items-center justify-between">
                <div>
                  <h1 className="text-2xl font-bold tracking-tight mb-1">
                    {categoryName ? `${categoryName} Projeleri` : "Yüklenenler"}
                  </h1>
                  <p className="text-muted-foreground text-sm">
                    {categoryName ? "Sadece bu kategoriye ait projeler listeleniyor." : "Sisteme yüklenen tüm projelerin durum listesi."}
                  </p>
                </div>
              </div>

              <div className="mb-6 flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <input
                    type="text"
                    placeholder="Proje ara..."
                    className="h-8 w-[250px] rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                  />
                </div>
              </div>

              <UploadsTable filterCategory={categoryName} />
            </div>
          </main>
        </div>
      </div>
    </AuthGuard>
  )
}
