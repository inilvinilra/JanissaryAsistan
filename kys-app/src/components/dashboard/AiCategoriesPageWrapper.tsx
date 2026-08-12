import React from "react"
import { AuthGuard, LogoutButton, UserInfo } from "@/components/auth/AuthGuard"
import { ThemeToggle } from "@/components/ui/theme-toggle"
import { Sidebar } from "@/components/dashboard/Sidebar"
import AiCategoriesPage from "@/components/dashboard/AiCategoriesPage"

export function AiCategoriesPageWrapper() {
  return (
    <AuthGuard>
      <div className="flex h-screen w-full bg-background text-foreground font-sans antialiased overflow-hidden">
        {/* Sol Menü */}
        <Sidebar />

        {/* Ana İçerik */}
        <div className="flex-1 flex flex-col min-w-0 overflow-hidden relative">
          
          {/* Üst Header */}
          <header className="h-16 border-b border-border/50 bg-card/30 backdrop-blur-md flex items-center justify-between px-6 shrink-0 sticky top-0 z-10">
            <div className="flex items-center gap-4">
              <div className="font-semibold text-lg bg-gradient-to-r from-primary to-blue-500 bg-clip-text text-transparent">
                JanissaryAsistan
              </div>
            </div>
            
            <div className="flex items-center gap-4">
              <UserInfo />
              <ThemeToggle />
              <LogoutButton />
            </div>
          </header>

          <AiCategoriesPage />
        </div>
      </div>
    </AuthGuard>
  )
}
