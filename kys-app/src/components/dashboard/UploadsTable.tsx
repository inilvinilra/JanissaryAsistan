import { useEffect, useState, useMemo } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { MoreHorizontal, CheckCircle2, Circle, HelpCircle, Trash2, Plus, Search, Download, Play, Loader2 } from "lucide-react"
import * as XLSX from "xlsx"

type UploadTask = {
  id: string;
  title: string;
  category: string;
  status: "done" | "todo" | "in-progress" | "canceled";
  grade: string;
  date: string;
  db_id?: string;
}

function gradeClass(grade: string) {
  if (grade.includes("A")) return "grade-a"
  if (grade.includes("B")) return "grade-b"
  if (grade.includes("C")) return "grade-c"
  if (grade === "-" || grade === "") return "grade-none"
  return "grade-f"
}

export function UploadsTable({ filterCategory }: { filterCategory?: string | null }) {
  const [tasks, setTasks] = useState<UploadTask[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState("")
  const [statusFilter, setStatusFilter] = useState<string>("all")
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [analyzingIds, setAnalyzingIds] = useState<Set<string>>(new Set())

  useEffect(() => {
    async function loadData() {
      try {
        const rawData: any[] = await tauriInvoke("get_recent_projects")
        const data = Array.isArray(rawData) ? rawData : []
        const dates = ["14 May 2026", "13 May 2026", "12 May 2026", "10 May 2026", "9 May 2026"]

        const formattedTasks: UploadTask[] = data.map((p, i) => {
          let status: UploadTask["status"] = "todo"
          if (p.score !== null && p.score !== undefined) status = "done"
          else if (p.status === "İnceleniyor") status = "in-progress"
          else if (p.status?.includes("Kopya") || p.status?.includes("Uyarı")) status = "done"

          return {
            id: p.id,
            title: p.title,
            category: p.category,
            status,
            grade: p.grade || "-",
            date: dates[i % dates.length],
            db_id: p.id,
          }
        })

        setTasks(formattedTasks)
      } catch (e) {
        console.error("Failed to load uploads", e)
        setTasks([])
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  // Arama + filtre
  const filtered = useMemo(() => {
    return tasks.filter(t => {
      const matchSearch =
        t.title.toLowerCase().includes(search.toLowerCase()) ||
        t.category.toLowerCase().includes(search.toLowerCase())
      const matchStatus =
        statusFilter === "all" ||
        (statusFilter === "done" && t.status === "done") ||
        (statusFilter === "in-progress" && t.status === "in-progress") ||
        (statusFilter === "todo" && t.status === "todo")
      const matchCat = filterCategory ? t.category === filterCategory : true
      return matchSearch && matchStatus && matchCat
    })
  }, [tasks, search, statusFilter, filterCategory])

  const toggleAll = () => {
    if (selectedIds.size === filtered.length) setSelectedIds(new Set())
    else setSelectedIds(new Set(filtered.map(t => t.id)))
  }

  const toggleOne = (id: string) => {
    const s = new Set(selectedIds)
    s.has(id) ? s.delete(id) : s.add(id)
    setSelectedIds(s)
  }

  const handleDelete = (id: string) => {
    if (confirm("Bu projeyi listeden kaldırmak istediğinize emin misiniz?")) {
      setTasks(tasks.filter(t => t.id !== id))
      setSelectedIds(prev => { const s = new Set(prev); s.delete(id); return s })
    }
  }

  const handleBulkDelete = () => {
    if (confirm(`${selectedIds.size} projeyi silmek istiyor musunuz?`)) {
      setTasks(tasks.filter(t => !selectedIds.has(t.id)))
      setSelectedIds(new Set())
    }
  }

  const handleAnalyze = async (db_id: string, file_path?: string) => {
    setAnalyzingIds(prev => { const s = new Set(prev); s.add(db_id); return s })
    try {
      await tauriInvoke("analyze_existing_project", { id: db_id })
      
      // Analiz bittikten sonra tablodaki tüm verileri baştan çek ki notlar ve durumlar güncellensin
      const rawData: any[] = await tauriInvoke("get_recent_projects")
      const data = Array.isArray(rawData) ? rawData : []
      const dates = ["14 May 2026", "13 May 2026", "12 May 2026", "10 May 2026", "9 May 2026"]

      const formattedTasks: UploadTask[] = data.map((p, i) => {
        let status: UploadTask["status"] = "todo"
        if (p.score !== null && p.score !== undefined) status = "done"
        else if (p.status === "İnceleniyor") status = "in-progress"
        else if (p.status?.includes("Kopya") || p.status?.includes("Uyarı")) status = "done"

        return {
          id: p.id,
          title: p.title,
          category: p.category,
          status,
          grade: p.grade || "-",
          date: dates[i % dates.length],
          db_id: p.id,
        }
      })
      setTasks(formattedTasks)
      
      // Bildirim göster ve detay sayfasına yönlendir
      alert("Analiz başarıyla tamamlandı! Detay sayfasına yönlendiriliyorsunuz...");
      window.location.href = `/project?id=${db_id}`;
    } catch (error) {
      console.error(error)
      alert("Analiz hatası: " + error)
    } finally {
      setAnalyzingIds(prev => { const s = new Set(prev); s.delete(db_id); return s })
    }
  }

  // XLSX Export
  const exportXLSX = (onlySelected = false) => {
    const toExport = onlySelected ? filtered.filter(t => selectedIds.has(t.id)) : filtered
    const data = toExport.map(t => ({
      ID: t.id,
      "Başlık": t.title,
      "Kategori": t.category,
      "Durum": t.status === "done" ? "İncelendi" : t.status === "todo" ? "Bekliyor" : t.status === "in-progress" ? "İnceleniyor" : "İptal",
      "Not": t.grade,
      "Tarih": t.date
    }))
    const ws = XLSX.utils.json_to_sheet(data)
    const wb = XLSX.utils.book_new()
    XLSX.utils.book_append_sheet(wb, ws, "Yüklemeler")
    XLSX.writeFile(wb, `janissary-projeler-${new Date().toISOString().slice(0,10)}.xlsx`)
  }

  if (loading) {
    return (
      <div className="space-y-2">
        {[1,2,3,4].map(i => (
          <div key={i} className="h-12 bg-card rounded-lg border border-border animate-pulse" />
        ))}
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-4 fade-in">
      {/* Toolbar */}
      <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center justify-between">
        <div className="flex items-center gap-2 flex-1 max-w-md">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
            <input
              value={search}
              onChange={e => setSearch(e.target.value)}
              placeholder="Proje veya kategori ara..."
              className="w-full pl-9 pr-3 h-9 bg-card border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors placeholder:text-muted-foreground"
            />
          </div>
          <select
            value={statusFilter}
            onChange={e => setStatusFilter(e.target.value)}
            className="h-9 bg-card border border-border rounded-lg text-sm px-3 focus:outline-none focus:ring-1 focus:ring-primary/50 text-foreground cursor-pointer"
          >
            <option value="all">Tümü</option>
            <option value="done">İncelendi</option>
            <option value="in-progress">İnceleniyor</option>
            <option value="todo">Bekliyor</option>
          </select>
        </div>

        <div className="flex items-center gap-2">
          {selectedIds.size > 0 && (
            <>
              <span className="text-xs text-muted-foreground">{selectedIds.size} seçili</span>
              <button
                onClick={() => exportXLSX(true)}
                className="h-8 px-3 text-xs bg-card border border-border rounded-lg hover:bg-muted transition-colors flex items-center gap-1.5"
              >
                <Download className="w-3.5 h-3.5" /> XLSX
              </button>
              <button
                onClick={handleBulkDelete}
                className="h-8 px-3 text-xs bg-red-500/10 text-red-500 border border-red-500/20 rounded-lg hover:bg-red-500/15 transition-colors"
              >
                Sil
              </button>
            </>
          )}
          <button
            onClick={() => exportXLSX(false)}
            className="h-8 px-3 text-xs bg-card border border-border rounded-lg hover:bg-muted transition-colors flex items-center gap-1.5 text-muted-foreground"
          >
            <Download className="w-3.5 h-3.5" /> Tümünü Dışa Aktar (XLSX)
          </button>
          <a
            href="/import"
            className="h-8 px-3 text-xs bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors flex items-center gap-1.5 font-medium"
          >
            <Plus className="w-3.5 h-3.5" /> Yeni Yükle
          </a>
        </div>
      </div>

      {/* Tablo */}
      <div className="rounded-xl border border-border bg-card overflow-hidden">
        {filtered.length === 0 ? (
          <div className="p-12 text-center text-muted-foreground text-sm">
            {search ? `"${search}" için sonuç bulunamadı` : "Henüz proje yüklenmemiş"}
          </div>
        ) : (
          <table className="w-full text-sm text-left">
            <thead className="border-b border-border bg-muted/30">
              <tr>
                <th className="h-10 px-4 w-10">
                  <input
                    type="checkbox"
                    checked={selectedIds.size === filtered.length && filtered.length > 0}
                    onChange={toggleAll}
                    className="accent-primary rounded"
                  />
                </th>
                <th className="h-10 px-4 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider w-[120px]">Tarih</th>
                <th className="h-10 px-4 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Proje</th>
                <th className="h-10 px-4 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider w-[150px]">Durum</th>
                <th className="h-10 px-4 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider w-[80px]">Not</th>
                <th className="h-10 px-4 w-20"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filtered.map(task => (
                <tr
                  key={task.id}
                  className={`hover:bg-muted/30 transition-colors group ${selectedIds.has(task.id) ? "bg-primary/5" : ""}`}
                >
                  <td className="px-4 py-3">
                    <input
                      type="checkbox"
                      checked={selectedIds.has(task.id)}
                      onChange={() => toggleOne(task.id)}
                      className="accent-primary rounded"
                    />
                  </td>
                  <td className="px-4 py-3 text-muted-foreground text-xs font-medium">{task.date}</td>
                  <td className="px-4 py-3">
                    <div className="flex flex-col gap-1.5">
                      <span className="font-medium text-foreground truncate max-w-xs">{task.title}</span>
                      <span className="text-[11px] font-medium px-2 py-0.5 rounded border border-border text-muted-foreground bg-muted/10 w-fit">
                        {task.category}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-1.5 text-muted-foreground">
                      {task.status === "done" && <CheckCircle2 className="w-3.5 h-3.5 text-emerald-500" />}
                      {task.status === "todo" && <Circle className="w-3.5 h-3.5" />}
                      {task.status === "in-progress" && <HelpCircle className="w-3.5 h-3.5 text-blue-400" />}
                      <span className="text-xs font-medium">
                        {task.status === "done" ? "İncelendi"
                          : task.status === "todo" ? "Bekliyor"
                          : task.status === "in-progress" ? "İnceleniyor"
                          : "İptal"}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`text-sm font-bold ${gradeClass(task.grade)}`}>
                      {task.grade}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                      {task.status !== "done" && task.db_id && (
                        <button
                          onClick={() => handleAnalyze(task.db_id as string)}
                          disabled={analyzingIds.has(task.db_id)}
                          className="h-7 px-2 flex items-center gap-1 rounded-md bg-primary/10 text-primary hover:bg-primary/20 transition-colors text-xs font-medium mr-1"
                        >
                          {analyzingIds.has(task.db_id) ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Play className="w-3.5 h-3.5" />}
                          Analiz Et
                        </button>
                      )}
                      {task.db_id && (
                        <a
                          href={`/project?id=${task.db_id}`}
                          className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-muted transition-colors text-muted-foreground"
                          title="Detay"
                        >
                          <MoreHorizontal className="w-4 h-4" />
                        </a>
                      )}
                      <button
                        onClick={() => handleDelete(task.id)}
                        className="w-7 h-7 flex items-center justify-center rounded-md hover:bg-red-500/10 hover:text-red-500 text-muted-foreground transition-colors"
                        title="Sil"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Footer bilgi */}
      <p className="text-xs text-muted-foreground">
        {filtered.length} / {tasks.length} proje gösteriliyor
        {search && ` — "${search}" araması`}
      </p>
    </div>
  )
}
