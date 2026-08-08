import React, { useEffect, useState } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { MoreHorizontal, CheckCircle2, Circle, HelpCircle, Trash2, Plus } from "lucide-react"

type UploadTask = {
  id: string; // e.g. PRJ-8782
  title: string;
  category: string;
  status: "done" | "todo" | "in-progress" | "canceled";
  grade: string;
  date: string;
  db_id?: string; // actual id for navigation
}

export function UploadsTable() {
  const [tasks, setTasks] = useState<UploadTask[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // Rust'tan verileri al
    async function loadData() {
      try {
        const rawData: any[] = await tauriInvoke("get_recent_projects")
        const data = Array.isArray(rawData) ? rawData : []; // Array.map Crash Önlemi
        const dates = ["14 Mayıs 2026", "13 Mayıs 2026", "12 Mayıs 2026", "10 Mayıs 2026", "9 Mayıs 2026"];
        
        // Veriyi Task Table formatına dönüştür
        const formattedTasks: UploadTask[] = data.map((p, i) => {
          let status: UploadTask["status"] = "todo";
          if (p.score !== null) status = "done";
          else if (p.status === "İnceleniyor") status = "in-progress";

          return {
            id: p.id,
            title: p.title,
            category: p.category,
            status,
            grade: p.grade || "-",
            date: dates[i % dates.length],
            db_id: p.id
          }
        })

        // Eğer boşsa mock data ekle
        if (formattedTasks.length === 0) {
          formattedTasks.push(
            { id: "PRJ-2041", title: "Görüntü İşleme ile Yüz Tanıma", category: "Yapay Zeka", status: "done", grade: "A", date: "14 Mayıs 2026", db_id: "PRJ-2041" },
            { id: "PRJ-2042", title: "Otonom Tarım Robotu", category: "Robotik", status: "done", grade: "B", date: "13 Mayıs 2026", db_id: "PRJ-2042" },
            { id: "PRJ-2043", title: "Akıllı Ev Güvenlik Sistemi", category: "Nesnelerin İnterneti", status: "todo", grade: "-", date: "12 Mayıs 2026", db_id: "PRJ-2043" },
            { id: "PRJ-2044", title: "Güneş Paneli Verimlilik Analizi", category: "Enerji", status: "in-progress", grade: "-", date: "10 Mayıs 2026", db_id: "PRJ-2044" }
          )
        }

        setTasks(formattedTasks)
      } catch (e) {
        console.error("Failed to load uploads", e)
        setTasks([]) // Çökmeyi engellemek için boş array
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  const handleDelete = (id: string) => {
    if (confirm("Bu projeyi silmek istediğinize emin misiniz?")) {
      setTasks(tasks.filter(t => t.id !== id))
    }
  }

  if (loading) {
    return <div className="p-12 text-center text-muted-foreground animate-pulse">Veriler yükleniyor...</div>
  }

  return (
    <div className="w-full flex flex-col font-sans text-sm">
      <div className="flex justify-between items-center mb-4">
        <div>
          {/* Header left area if needed */}
        </div>
        <a href="/import" className="inline-flex items-center gap-2 bg-primary text-primary-foreground hover:bg-primary/90 px-4 py-2 rounded-md text-sm font-medium transition-colors">
          <Plus className="w-4 h-4" /> Yeni Proje Yükle
        </a>
      </div>

      <div className="rounded-md border border-border bg-card">
        <table className="w-full text-left border-collapse">
          <thead className="border-b border-border text-muted-foreground bg-muted/30">
            <tr>
              <th className="h-10 px-4 align-middle font-medium w-12">
                <input type="checkbox" className="rounded border-input bg-transparent accent-primary" />
              </th>
              <th className="h-10 px-4 align-middle font-medium w-[120px]">Tarih</th>
              <th className="h-10 px-4 align-middle font-medium">Başlık</th>
              <th className="h-10 px-4 align-middle font-medium w-[150px]">Durum</th>
              <th className="h-10 px-4 align-middle font-medium w-[120px]">Harf Notu</th>
              <th className="h-10 px-4 align-middle font-medium w-24">İşlemler</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-border">
            {tasks.map((task) => (
              <tr key={task.id} className="hover:bg-muted/50 transition-colors group">
                <td className="px-4 py-3 align-middle">
                  <input type="checkbox" className="rounded border-input bg-transparent accent-primary" />
                </td>
                <td className="px-4 py-3 align-middle text-muted-foreground font-medium">
                  {task.date}
                </td>
                <td className="px-4 py-3 align-middle">
                  <div className="flex items-center gap-2">
                    <span className="inline-flex items-center rounded-full border border-border px-2.5 py-0.5 text-xs font-semibold text-muted-foreground transition-colors hover:bg-secondary">
                      {task.category}
                    </span>
                    <span className="truncate max-w-[500px] text-foreground font-medium">
                      {task.title}
                    </span>
                  </div>
                </td>
                <td className="px-4 py-3 align-middle">
                  <div className="flex items-center gap-2 text-muted-foreground">
                    {task.status === "done" && <CheckCircle2 className="w-4 h-4 text-green-500" />}
                    {task.status === "todo" && <Circle className="w-4 h-4 text-muted-foreground" />}
                    {task.status === "in-progress" && <HelpCircle className="w-4 h-4 text-blue-500" />}
                    {task.status === "canceled" && <Circle className="w-4 h-4 opacity-50" />}
                    
                    <span className="capitalize font-medium">
                      {task.status === "done" ? "İncelendi" : task.status === "todo" ? "İncelenmedi" : task.status === "in-progress" ? "İnceleniyor" : "İptal Edildi"}
                    </span>
                  </div>
                </td>
                <td className="px-4 py-3 align-middle">
                  <div className="flex items-center gap-2">
                    <span className={`inline-flex items-center justify-center font-bold text-[11px] rounded w-6 h-6 border ${
                      task.grade.includes('A') ? 'bg-emerald-50 text-emerald-700 border-emerald-200 dark:bg-emerald-950/30' :
                      task.grade.includes('B') ? 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950/30' :
                      task.grade.includes('C') ? 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950/30' :
                      task.grade === '-' ? 'bg-muted text-muted-foreground border-border' :
                      'bg-rose-50 text-rose-700 border-rose-200 dark:bg-rose-950/30'
                    }`}>
                      {task.grade}
                    </span>
                  </div>
                </td>
                <td className="px-4 py-3 align-middle text-right">
                  <div className="flex items-center justify-end gap-1">
                    {task.db_id && (
                      <a href={`/project?id=${task.db_id}`} title="Detaylara Git" className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors hover:bg-accent hover:text-accent-foreground h-8 w-8">
                        <MoreHorizontal className="w-4 h-4" />
                      </a>
                    )}
                    <button 
                      onClick={() => handleDelete(task.id)}
                      title="Sil" 
                      className="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors hover:bg-red-500/10 hover:text-red-500 text-muted-foreground h-8 w-8"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
