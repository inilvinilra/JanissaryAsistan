import React, { useState, useEffect } from "react"
import {
  LayoutDashboard, Upload, ClipboardList, Settings,
  ChevronLeft, ChevronRight, Folder, ChevronDown, BrainCircuit
} from "lucide-react"
import { supabase } from "@/lib/supabase"
import logoImg from "@/assets/jannisary.png"
import logoWhiteImg from "@/assets/beyaz.png"

type NavItem = {
  label: string;
  icon: any;
  href: string;
}

const NAV_ITEMS: NavItem[] = [
  { label: "Dashboard", icon: LayoutDashboard, href: "/dashboard" },
  { label: "PDF Yükle", icon: Upload, href: "/import" },
  { label: "Yüklenenler", icon: ClipboardList, href: "/uploads" },
  { label: "Kategori & AI Yönetimi", icon: BrainCircuit, href: "/settings?tab=kategoriler" },
  { label: "Ayarlar", icon: Settings, href: "/settings" },
]

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(false)
  const [categoriesOpen, setCategoriesOpen] = useState(false)
  const [categories, setCategories] = useState<any[]>([])
  const [projects, setProjects] = useState<any[]>([])
  const [expandedCat, setExpandedCat] = useState<string | null>(null)
  const currentPath = typeof window !== "undefined" ? window.location.pathname : ""

  useEffect(() => {
    async function loadData() {
      // 1. Kategorileri çek
      const { data: catData } = await supabase.from("evaluation_categories").select("id, name").order("name")
      if (catData) setCategories(catData)
      
      // 2. Projeleri çek (başlık ve kategori)
      try {
        const res = await fetch("http://localhost:8080/api/projects")
        if (res.ok) {
           const json = await res.json()
           if (json.status === "success") setProjects(Array.isArray(json.data) ? json.data : [])
        } else {
           // fallback to supabase if axum is not responding
           const { data: projData } = await supabase.from("projects").select("id, filename, category")
           if (projData) setProjects(Array.isArray(projData) ? projData : [])
        }
      } catch(e) {
        const { data: projData } = await supabase.from("projects").select("id, filename, category")
        if (projData) setProjects(Array.isArray(projData) ? projData : [])
      }
    }
    loadData()
  }, [])

  return (
    <aside className={`flex flex-col border-r border-border bg-card transition-all duration-300 shrink-0 ${collapsed ? "w-16" : "w-64"}`}>
      {/* Logo */}
      <div className={`h-20 flex items-center border-b border-border shrink-0 overflow-hidden ${collapsed ? "justify-center px-0" : "px-4 gap-2"}`}>
        <img src={typeof logoImg === "string" ? logoImg : (logoImg as any).src} alt="Janissary Logo" className="h-16 w-auto object-contain shrink-0 dark:hidden" />
        <img src={typeof logoWhiteImg === "string" ? logoWhiteImg : (logoWhiteImg as any).src} alt="Janissary Logo White" className="h-16 w-auto object-contain shrink-0 hidden dark:block" />
        {!collapsed && (
          <div className="flex-1 min-w-0">
            <span className="text-base font-bold tracking-tight block truncate">JanissaryAsistan</span>
            <span className="text-[11px] text-muted-foreground block font-medium truncate">Jüri Paneli</span>
          </div>
        )}
      </div>

      {/* Navigasyon */}
      <nav className="flex-1 p-2 flex flex-col gap-0.5 overflow-y-auto custom-scrollbar">
        {NAV_ITEMS.map(item => {
          const Icon = item.icon
          const active = currentPath === item.href || currentPath.startsWith(item.href + "/")

          return (
            <a
              key={item.href}
              href={item.href}
              title={collapsed ? item.label : undefined}
              className={`flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm font-medium transition-all duration-150
                ${active
                  ? "sidebar-active"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
                }
                ${collapsed ? "justify-center" : ""}`}
            >
              <Icon className={`shrink-0 ${collapsed ? "w-5 h-5" : "w-4 h-4"}`} />
              {!collapsed && <span className="truncate">{item.label}</span>}
            </a>
          )
        })}

        {/* Dinamik Kategoriler Menüsü */}
        {categories.length > 0 && (
          <div className="mt-1 flex flex-col gap-0.5">
            <button
              onClick={() => {
                if (collapsed) setCollapsed(false);
                setCategoriesOpen(!categoriesOpen);
              }}
              className={`flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm font-medium transition-all duration-150 text-muted-foreground hover:bg-muted hover:text-foreground ${collapsed ? "justify-center" : ""}`}
              title={collapsed ? "Kategoriler" : undefined}
            >
              <Folder className={`shrink-0 ${collapsed ? "w-5 h-5" : "w-4 h-4"}`} />
              {!collapsed && (
                <div className="flex flex-1 items-center justify-between">
                  <span>Kategoriler</span>
                  <ChevronDown className={`w-3 h-3 transition-transform ${categoriesOpen ? "rotate-180" : ""}`} />
                </div>
              )}
            </button>
            
            {/* Alt Kategoriler Listesi */}
            {(!collapsed && categoriesOpen) && (
              <div className="pl-9 pr-2 py-1 flex flex-col gap-1">
                {categories.map(cat => {
                  const catProjects = projects.filter(p => p.category === cat.name)
                  const isExpanded = expandedCat === cat.name
                  
                  return (
                    <div key={cat.id} className="flex flex-col">
                      <div className="flex items-center justify-between group">
                        <a
                          href={`/dashboard?category=${encodeURIComponent(cat.name)}`}
                          className="flex-1 text-xs font-medium text-muted-foreground group-hover:text-foreground py-1.5 px-2 rounded-md transition-colors truncate"
                        >
                          {cat.name}
                        </a>
                        {catProjects.length > 0 && (
                          <button
                            onClick={(e) => { e.preventDefault(); setExpandedCat(isExpanded ? null : cat.name) }}
                            className="p-1 rounded-md text-muted-foreground hover:bg-muted"
                          >
                            <ChevronDown className={`w-3 h-3 transition-transform ${isExpanded ? "rotate-180" : ""}`} />
                          </button>
                        )}
                      </div>
                      
                      {/* Kategori İçi Projeler */}
                      {isExpanded && catProjects.length > 0 && (
                        <div className="pl-2 pr-1 py-1 flex flex-col gap-1 border-l border-border ml-2 mt-1">
                          {catProjects.map(p => (
                            <a
                              key={p.id}
                              href={`/project?id=PRJ-${p.id}`}
                              className="text-[11px] text-muted-foreground/80 hover:text-foreground py-1 px-2 rounded hover:bg-muted/50 truncate transition-colors"
                              title={p.filename || p.title}
                            >
                              {p.filename || p.title}
                            </a>
                          ))}
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        )}
      </nav>

      {/* Alt: collapse toggle */}
      <div className="p-2 border-t border-border">
        <button
          onClick={() => setCollapsed(!collapsed)}
          className={`w-full flex items-center gap-3 px-2.5 py-2 rounded-lg text-sm text-muted-foreground hover:bg-muted hover:text-foreground transition-all ${collapsed ? "justify-center" : ""}`}
          title={collapsed ? "Genişlet" : "Daralt"}
        >
          {collapsed
            ? <ChevronRight className="w-4 h-4" />
            : <><ChevronLeft className="w-4 h-4" /><span className="text-xs">Daralt</span></>
          }
        </button>
      </div>
    </aside>
  )
}
