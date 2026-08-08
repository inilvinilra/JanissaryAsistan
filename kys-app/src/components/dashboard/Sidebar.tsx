import React, { useState } from "react"
import { 
  Brain, Shield, Database, GraduationCap, Activity, 
  Sigma, Atom, Bot, FlaskConical, Code, Leaf, Cpu, 
  Menu, ChevronLeft, LayoutDashboard, Settings, FileUp, ListTodo
} from "lucide-react"

const categories = [
  { name: "Veri Bilimi", icon: Database, color: "text-blue-500", active: true },
  { name: "Yapay Zeka", icon: Brain, color: "text-purple-500", active: false },
  { name: "Siber Güvenlik", icon: Shield, color: "text-red-500", active: false },
  { name: "Eğitim Tech", icon: GraduationCap, color: "text-green-500", active: false },
  { name: "Sağlık Tech", icon: Activity, color: "text-rose-500", active: false },
  { name: "Matematik", icon: Sigma, color: "text-indigo-500", active: false },
  { name: "Fizik/Kimya", icon: Atom, color: "text-yellow-500", active: false },
  { name: "Robotik", icon: Bot, color: "text-orange-500", active: false },
  { name: "Biyoloji", icon: FlaskConical, color: "text-teal-500", active: false },
  { name: "Yazılım/Oyun", icon: Code, color: "text-cyan-500", active: false },
  { name: "Tarım Tech", icon: Leaf, color: "text-emerald-500", active: false },
  { name: "Donanım", icon: Cpu, color: "text-slate-500", active: false },
]

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(false)

  return (
    <div className={`h-full bg-card border-r border-border transition-all duration-300 flex flex-col shrink-0 overflow-hidden ${collapsed ? 'w-20' : 'w-64'}`}>
      
      {/* Brand & Toggle */}
      <div className="h-16 flex items-center justify-between px-4 border-b border-border shrink-0">
        {!collapsed && (
          <div className="flex items-center gap-2 overflow-hidden">
            <div className="w-8 h-8 rounded bg-primary text-primary-foreground flex items-center justify-center font-bold text-lg shrink-0">
              K
            </div>
            <span className="font-bold text-lg whitespace-nowrap">KYS Engine</span>
          </div>
        )}
        {collapsed && (
          <div className="w-8 h-8 mx-auto rounded bg-primary text-primary-foreground flex items-center justify-center font-bold text-lg shrink-0">
            K
          </div>
        )}
        <button 
          onClick={() => setCollapsed(!collapsed)}
          className={`p-1.5 rounded-lg hover:bg-muted text-muted-foreground transition-colors ${collapsed ? 'mx-auto mt-4 absolute top-2 right-4' : ''}`}
        >
          {collapsed ? <Menu className="w-5 h-5" /> : <ChevronLeft className="w-5 h-5" />}
        </button>
      </div>

      <div className="flex-1 overflow-y-auto custom-scrollbar py-6 flex flex-col gap-8">
        
        {/* Navigation */}
        <div className="px-3">
          {!collapsed && <h3 className="px-3 text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">KYS Sistemi</h3>}
          <div className="space-y-1">
            <a href="/dashboard" className="flex items-center gap-3 px-3 py-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors">
              <LayoutDashboard className="w-5 h-5 shrink-0" />
              {!collapsed && <span className="font-medium">Ana Dashboard</span>}
            </a>
            <a href="/import" className="flex items-center gap-3 px-3 py-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors">
              <FileUp className="w-5 h-5 shrink-0" />
              {!collapsed && <span className="font-medium">Proje Yükle</span>}
            </a>
            <a href="/uploads" className="flex items-center gap-3 px-3 py-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors">
              <ListTodo className="w-5 h-5 shrink-0" />
              {!collapsed && <span className="font-medium">Yüklenenler</span>}
            </a>
            <a href="#" className="flex items-center gap-3 px-3 py-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted transition-colors">
              <Settings className="w-5 h-5 shrink-0" />
              {!collapsed && <span className="font-medium">Ayarlar</span>}
            </a>
          </div>
        </div>

        {/* Categories (ALANLAR) */}
        <div className="px-3">
          {!collapsed && (
            <h4 className="text-[11px] font-bold text-muted-foreground uppercase tracking-wider mb-2 px-3">
              Alanlar
            </h4>
          )}
          <div className="space-y-0.5">
            {categories.map((cat, idx) => {
              const Icon = cat.icon
              return (
                <button 
                  key={idx}
                  className={`w-full flex items-center justify-between px-3 py-2 rounded-lg transition-colors group ${
                    cat.active 
                      ? "bg-primary/10 text-primary" 
                      : "text-muted-foreground hover:bg-muted hover:text-foreground"
                  }`}
                  title={collapsed ? cat.name : undefined}
                >
                  <div className="flex items-center gap-3">
                    <Icon className={`w-4 h-4 shrink-0 ${cat.active ? "text-primary" : "text-muted-foreground group-hover:text-foreground"}`} />
                    {!collapsed && <span className="text-sm font-medium truncate">{cat.name}</span>}
                  </div>
                  {/* Count was removed, removing the badge */}
                </button>
              )
            })}
          </div>
        </div>
      </div>
      
      {/* Footer Nav */}
      <div className="p-3 border-t border-border">
        <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-colors hover:bg-muted text-muted-foreground">
          <Settings className="w-5 h-5 shrink-0" />
          {!collapsed && <span className="font-medium text-sm">Ayarlar</span>}
        </button>
      </div>
    </div>
  )
}
