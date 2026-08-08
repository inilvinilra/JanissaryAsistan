import React, { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { MoreHorizontal, ArrowUp, Download, Trash2, FileOutput } from "lucide-react"

type Project = {
  id: string;
  title: string;
  category: string;
  author?: string;
  score: number | null;
  grade: string;
  status: string;
}

export function RecentProjectsTable() {
  const [projects, setProjects] = useState<Project[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())

  useEffect(() => {
    async function loadData() {
      try {
        const rawData: Project[] = await tauriInvoke("get_recent_projects")
        const data = Array.isArray(rawData) ? rawData : []; // Crash önleme (Güvenli dizi kontrolü)
        
        // Mock data çoğaltma (kaydırma - scroll test etmek için 20 elemana çıkaralım)
        let expandedData: Project[] = [];
        for(let i=0; i<4; i++) {
          expandedData = [...expandedData, ...data.map(d => ({...d, id: `${d.id}-${i}`}))];
        }

        const sorted = expandedData.sort((a, b) => (b.score || 0) - (a.score || 0))
        
        const withAuthors = sorted.map((p, i) => ({
          ...p,
          author: p.author || ["Ahmet Yılmaz", "Ayşe Demir", "Mehmet Kaya", "Fatma Çelik", "Ali Veli"][i % 5]
        }))
        
        setProjects(withAuthors)
      } catch (e) {
        console.error("Failed to load projects", e)
        setProjects([]) // Hata durumunda boş dizi kalsın, çökmesin.
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  const toggleSelectAll = () => {
    if (selectedIds.size === projects.length) {
      setSelectedIds(new Set())
    } else {
      setSelectedIds(new Set(projects.map(p => p.id)))
    }
  }

  const toggleSelect = (id: string) => {
    const newSet = new Set(selectedIds)
    if (newSet.has(id)) newSet.delete(id)
    else newSet.add(id)
    setSelectedIds(newSet)
  }

  const handleDeleteSelected = () => {
    if(confirm(`${selectedIds.size} projeyi silmek istediğinize emin misiniz?`)) {
      setProjects(projects.filter(p => !selectedIds.has(p.id)))
      setSelectedIds(new Set())
    }
  }

  if (loading) {
    return <div className="mt-8 p-12 text-center text-muted-foreground animate-pulse border border-border bg-card rounded-xl">Analiz Verileri Yükleniyor...</div>
  }

  return (
    <div className="flex flex-col mt-8 gap-4 font-sans">
      
      {/* Action Bar (Toplu İşlemler) */}
      {selectedIds.size > 0 && (
        <div className="bg-primary/10 border border-primary/20 rounded-xl p-3 flex items-center justify-between shadow-sm animate-in fade-in slide-in-from-bottom-2">
          <span className="text-primary font-medium text-sm ml-2">
            {selectedIds.size} proje seçildi
          </span>
          <div className="flex items-center gap-2">
            <button className="bg-background border border-border text-foreground px-4 py-1.5 rounded-lg text-sm font-medium hover:bg-muted flex items-center gap-2 transition-colors">
              <FileOutput className="w-4 h-4" /> Rapor Oluştur
            </button>
            <button className="bg-primary text-primary-foreground px-4 py-1.5 rounded-lg text-sm font-medium hover:bg-primary/90 flex items-center gap-2 transition-colors">
              <Download className="w-4 h-4" /> Dışa Aktar (CSV)
            </button>
            <button onClick={handleDeleteSelected} className="bg-red-500/10 text-red-600 px-4 py-1.5 rounded-lg text-sm font-medium hover:bg-red-500/20 flex items-center gap-2 transition-colors ml-2">
              <Trash2 className="w-4 h-4" /> Sil
            </button>
          </div>
        </div>
      )}

      <div className="bg-card text-foreground rounded-xl shadow-sm border border-border overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm text-left border-collapse">
          <thead className="text-[11px] text-muted-foreground uppercase bg-muted/50 border-b border-border tracking-wider">
            <tr>
              <th scope="col" className="px-4 py-3 font-semibold w-10">
                <input 
                  type="checkbox" 
                  className="rounded border-input text-primary focus:ring-primary accent-primary"
                  checked={selectedIds.size === projects.length && projects.length > 0}
                  onChange={toggleSelectAll}
                />
              </th>
              <th scope="col" className="px-4 py-3 font-medium truncate max-w-[250px]">Proje Adı</th>
              <th scope="col" className="px-4 py-3 font-medium">Durum</th>
              <th scope="col" className="px-4 py-3 font-medium text-right group cursor-pointer hover:text-foreground transition-colors">
                <div className="flex items-center justify-end gap-1">
                  Gönderen
                  <MoreHorizontal className="w-3 h-3 opacity-0 group-hover:opacity-100" />
                </div>
              </th>
              <th scope="col" className="px-4 py-3 font-medium text-right cursor-pointer hover:text-foreground transition-colors">
                <div className="flex items-center justify-end gap-1">
                  KYS Puanı <ArrowUp className="w-3 h-3 text-blue-500" />
                </div>
              </th>
              <th scope="col" className="px-4 py-3 font-medium text-right">Kategori</th>
              <th scope="col" className="px-4 py-3 font-medium text-right">Analiz Detayı</th>
              <th scope="col" className="px-4 py-3 font-medium w-32">Gelişim / Risk</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {projects.map((project) => (
              <tr 
                key={project.id} 
                className={`hover:bg-muted/50 transition-colors ${selectedIds.has(project.id) ? 'bg-primary/5 hover:bg-primary/10' : ''}`}
              >
                <td className="px-4 py-3.5">
                  <input 
                    type="checkbox" 
                    className="rounded border-input text-primary focus:ring-primary accent-primary"
                    checked={selectedIds.has(project.id)}
                    onChange={() => toggleSelect(project.id)}
                  />
                </td>
                <td className="px-4 py-3.5 font-medium truncate max-w-[250px]">{project.title}</td>
                <td className="px-4 py-3.5">
                  <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-bold uppercase tracking-wider border ${
                    project.score !== null ? "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20" :
                    project.status === "İnceleniyor" ? "bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20" :
                    "bg-muted text-muted-foreground border-border"
                  }`}>
                    {project.score !== null ? "İncelendi" : project.status === "İnceleniyor" ? "İnceleniyor" : "İncelenmedi"}
                  </span>
                </td>
                <td className="px-4 py-3.5 text-right font-medium">{project.author}</td>
                <td className="px-4 py-3.5 text-right font-semibold tabular-nums">{project.score || "-"}</td>
                <td className="px-4 py-3.5 text-right text-muted-foreground">{project.category}</td>
                <td className="px-4 py-3.5 text-right">
                  <a href={`/project?id=${project.id}`} className="text-xs bg-muted hover:bg-muted/80 border border-border text-foreground px-3 py-1 rounded transition-colors">
                    Detay
                  </a>
                </td>
                <td className="px-4 py-3.5">
                  <Sparkline score={project.score} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

// Görseldeki bar chart (Sparkline) tasarımını taklit eden basit bileşen
function Sparkline({ score }: { score: number | null }) {
  if (!score) return <div className="h-4 flex items-end gap-0.5"><div className="w-1.5 h-1 bg-muted"></div></div>
  
  // Sahte bir bar dizisi oluştur, skor ne kadar yüksekse barlar o kadar artan eğilime sahip olsun
  const bars = Array.from({ length: 15 }).map((_, i) => {
    const baseHeight = (score / 100) * 16; 
    const randomVariation = Math.random() * 4 - 2;
    // Giderek artan bir trend görünümü
    const height = Math.max(2, Math.min(16, baseHeight * (0.5 + (i / 15) * 0.8) + randomVariation));
    return height;
  });

  return (
    <div className="h-4 flex items-end gap-0.5 opacity-80">
      {bars.map((h, i) => (
        <div key={i} className="w-1 bg-blue-500 rounded-t-[1px]" style={{ height: `${h}px` }} />
      ))}
    </div>
  )
}
