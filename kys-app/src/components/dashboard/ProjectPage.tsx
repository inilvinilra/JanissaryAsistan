import React from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import { ProjectDetailView } from "@/components/dashboard/ProjectDetail"

export function ProjectPage() {
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
              <span className="text-foreground">JanissaryAsistan</span> / Proje Detayı
            </div>
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>

          {/* Kaydırılabilir Ana Alan */}
          <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar bg-background">
            <div className="max-w-[1600px] mx-auto h-full">
              <ProjectDetailView />
            </div>
          </main>
        </div>
      </div>
    </AuthGuard>
  )
}
