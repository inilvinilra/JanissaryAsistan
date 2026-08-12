import React from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import { SettingsPage as SettingsContent } from "@/components/dashboard/SettingsContent"

export function SettingsPage() {
  return (
    <AuthGuard>
      <div className="flex h-screen w-full bg-background text-foreground font-sans antialiased overflow-hidden">
        <Sidebar />
        <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
          <header className="h-20 border-b border-border bg-card/50 flex items-center px-6 justify-between shrink-0">
            <div className="font-semibold text-lg text-muted-foreground flex items-center gap-2">
              <span className="text-foreground">JanissaryAsistan</span> / Ayarlar
            </div>
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>
          <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar bg-background">
            <div className="max-w-3xl mx-auto">
              <div className="mb-6">
                <h1 className="text-2xl font-bold tracking-tight">Ayarlar</h1>
                <p className="text-sm text-muted-foreground mt-1">Hesap, sistem bağlantısı ve veri yönetimi ayarları.</p>
              </div>
              <SettingsContent />
            </div>
          </main>
        </div>
      </div>
    </AuthGuard>
  )
}
