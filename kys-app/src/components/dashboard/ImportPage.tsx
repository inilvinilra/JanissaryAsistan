import React from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import { FileUploader } from "@/components/dashboard/FileUploader"

export function ImportPage() {
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
              <span className="text-foreground">JanissaryAsistan</span> / Proje (PDF) Yükle
            </div>
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>

          {/* Ana Alan */}
          <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar bg-background flex flex-col">
            <div className="max-w-3xl mx-auto w-full flex-1 flex flex-col justify-center">
              <div className="mb-8 text-center">
                <h1 className="text-3xl font-bold tracking-tight mb-2">Yeni Proje Analizi Başlat</h1>
                <p className="text-muted-foreground">JanissaryAsistan Engine üzerinde analiz edilmesi için proje raporunu (PDF) yükleyin.</p>
              </div>

              <FileUploader />

              <div className="mt-8 bg-blue-500/10 border border-blue-500/20 rounded-xl p-4 flex gap-3 text-sm text-blue-700 dark:text-blue-200">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="shrink-0 text-blue-400 mt-0.5">
                  <circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>
                </svg>
                <div>
                  <strong className="block mb-1 text-blue-600 dark:text-blue-400">Nasıl Çalışır?</strong>
                  Yüklediğiniz PDF dosyası Rust tabanlı JanissaryAsistan Engine motoruna iletilir. Doğal dil işleme, referans analizi ve özgünlük (kopya) kontrolünden geçtikten sonra detaylı skoruyla birlikte sisteme kaydedilir.
                </div>
              </div>
            </div>
          </main>
        </div>
      </div>
    </AuthGuard>
  )
}
