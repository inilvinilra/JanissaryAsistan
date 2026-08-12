import { useEffect, useState, useMemo } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { ArrowUp, Download, FileOutput, Search, ChevronUp, ChevronDown, FileText, GripVertical } from "lucide-react"
import * as XLSX from "xlsx"

type Project = {
  id: string;
  title: string;
  category: string;
  author?: string;
  score?: number | null;
  grade: string;
  status: string;
  word_count: number;
}

type SortKey = "custom" | "score" | "title" | "grade" | "status" | "word_count"
type SortDir = "asc" | "desc"

function gradeClass(grade: string) {
  if (grade.includes("A")) return "grade-a"
  if (grade.includes("B")) return "grade-b"
  if (grade.includes("C")) return "grade-c"
  if (grade === "-" || grade === "") return "grade-none"
  return "grade-f"
}


export function RecentProjectsTable() {
  const [projects, setProjects] = useState<Project[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set())
  const [search, setSearch] = useState("")
  const [categoryFilter, setCategoryFilter] = useState("Tümü")
  const [sortKey, setSortKey] = useState<SortKey>("custom")
  const [sortDir, setSortDir] = useState<SortDir>("desc")
  const [draggedItemId, setDraggedItemId] = useState<string | null>(null)
  const [dragOverItemId, setDragOverItemId] = useState<string | null>(null)
  const [customOrder, setCustomOrder] = useState<string[]>([])
  const [draggableId, setDraggableId] = useState<string | null>(null)

  useEffect(() => {
    // URL'den kategori parametresi varsa al
    if (typeof window !== "undefined") {
      const urlParams = new URLSearchParams(window.location.search);
      const cat = urlParams.get("category");
      if (cat) {
        setCategoryFilter(cat);
      }
      
      const savedOrder = localStorage.getItem("janissary_custom_order")
      if (savedOrder) {
        try { setCustomOrder(JSON.parse(savedOrder)) } catch (e) {}
      }
    }

    async function loadData() {
      try {
        const rawData: Project[] = await tauriInvoke("get_recent_projects")
        const data = Array.isArray(rawData) ? rawData : []

        const formatted = data.map((p) => ({
          ...p,
          author: p.author || "Bilinmeyen Kullanıcı",
        }))

        setProjects(formatted)
      } catch (e) {
        console.error("Failed to load projects", e)
        setProjects([])
      } finally {
        setLoading(false)
      }
    }
    loadData()
  }, [])

  const toggleSelectAll = () => {
    if (selectedIds.size === filtered.length) setSelectedIds(new Set())
    else setSelectedIds(new Set(filtered.map(p => p.id)))
  }

  const toggleSelect = (id: string) => {
    const s = new Set(selectedIds)
    s.has(id) ? s.delete(id) : s.add(id)
    setSelectedIds(s)
  }

  const handleDeleteSelected = () => {
    if (confirm(`${selectedIds.size} projeyi silmek istediğinize emin misiniz?`)) {
      setProjects(projects.filter(p => !selectedIds.has(p.id)))
      setSelectedIds(new Set())
    }
  }

  const handleSort = (key: SortKey) => {
    if (sortKey === key) setSortDir(d => d === "asc" ? "desc" : "asc")
    else { setSortKey(key); setSortDir("desc") }
  }

  // Drag and Drop Handlers
  const handleDragStart = (e: React.DragEvent, id: string) => {
    setDraggedItemId(id)
    e.dataTransfer.effectAllowed = "move"
    if (sortKey !== "custom") setSortKey("custom")
  }

  const handleDragEnter = (e: React.DragEvent, id: string) => {
    e.preventDefault()
    setDragOverItemId(id)
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
  }

  const handleDrop = (e: React.DragEvent, targetId: string) => {
    e.preventDefault()
    if (!draggedItemId || draggedItemId === targetId) {
      setDraggedItemId(null)
      setDragOverItemId(null)
      return
    }

    const items = [...filtered]
    const draggedIndex = items.findIndex(i => i.id === draggedItemId)
    const targetIndex = items.findIndex(i => i.id === targetId)

    if (draggedIndex !== -1 && targetIndex !== -1) {
      const [draggedItem] = items.splice(draggedIndex, 1)
      items.splice(targetIndex, 0, draggedItem)
      
      const newOrder = items.map(i => i.id)
      
      // Projeler listesinde olmayan ID'leri de customOrder'da sakla (kaybolmasın diye)
      const mergedOrder = Array.from(new Set([...newOrder, ...customOrder]))

      setCustomOrder(mergedOrder)
      if (typeof window !== "undefined") {
        localStorage.setItem("janissary_custom_order", JSON.stringify(mergedOrder))
      }

      // Backend'e sıralamayı gönder
      try {
        const ranks = mergedOrder.map((id, index) => ({ id, rank: index + 1 }))
        fetch("http://localhost:8080/api/projects/reorder", {
          method: "PUT",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ ranks })
        })
      } catch (e) {
        console.error("Sıralama backend'e iletilemedi:", e)
      }
    }
    setDraggedItemId(null)
    setDragOverItemId(null)
  }

  const handleDragEnd = () => {
    setDraggedItemId(null)
    setDragOverItemId(null)
  }

  // XLSX Export
  const exportXLSX = () => {
    const toExport = selectedIds.size > 0
      ? filtered.filter(p => selectedIds.has(p.id))
      : filtered
    
    const data = toExport.map(p => ({
      ID: p.id,
      "Başlık": p.title,
      "Kategori": p.category,
      "Gönderen": p.author || "",
      "Kelime": p.word_count,
      "Puan": p.score ?? "-",
      "Not": p.grade,
      "Durum": p.status
    }))

    const ws = XLSX.utils.json_to_sheet(data)
    const wb = XLSX.utils.book_new()
    XLSX.utils.book_append_sheet(wb, ws, "Projeler")
    XLSX.writeFile(wb, `janissary-analiz-${new Date().toISOString().slice(0,10)}.xlsx`)
  }

  // Filtre + sıralama
  const filtered = useMemo(() => {
    let result = projects.filter(p => {
      const pCategory = p.category || "Genel"
      const pTitle = p.title || ""
      const pAuthor = p.author || ""
      
      return (categoryFilter === "Tümü" || pCategory === categoryFilter) &&
      (pTitle.toLowerCase().includes(search.toLowerCase()) ||
       pCategory.toLowerCase().includes(search.toLowerCase()) ||
       pAuthor.toLowerCase().includes(search.toLowerCase()))
    })

    result.sort((a, b) => {
      if (sortKey === "custom") {
        const indexA = customOrder.indexOf(a.id)
        const indexB = customOrder.indexOf(b.id)
        if (indexA === -1 && indexB === -1) return 0
        if (indexA === -1) return 1
        if (indexB === -1) return -1
        return indexA - indexB
      }

      let va: any, vb: any
      if (sortKey === "score") { va = a.score ?? -1; vb = b.score ?? -1 }
      else if (sortKey === "title") { va = a.title; vb = b.title }
      else if (sortKey === "grade") { va = a.grade; vb = b.grade }
      else if (sortKey === "word_count") { va = a.word_count; vb = b.word_count }
      else { va = a.status; vb = b.status }

      if (va < vb) return sortDir === "asc" ? -1 : 1
      if (va > vb) return sortDir === "asc" ? 1 : -1
      return 0
    })

    return result
  }, [projects, search, categoryFilter, sortKey, sortDir, customOrder])

  // Dinamik kategorileri bul (ve url'den gelen filtreyi de ekle)
  const categories = useMemo(() => {
    const cats = new Set(projects.map(p => p.category))
    if (categoryFilter !== "Tümü") cats.add(categoryFilter)
    return ["Tümü", ...Array.from(cats).sort()].filter(Boolean)
  }, [projects, categoryFilter])

  function SortIcon({ col }: { col: SortKey }) {
    if (sortKey !== col) return <ArrowUp className="w-3 h-3 opacity-20" />
    return sortDir === "desc"
      ? <ChevronDown className="w-3 h-3 text-primary" />
      : <ChevronUp className="w-3 h-3 text-primary" />
  }

  if (loading) {
    return (
      <div className="space-y-2 mt-8">
        {[1,2,3,4,5].map(i => (
          <div key={i} className="h-12 bg-card rounded-lg border border-border animate-pulse" />
        ))}
      </div>
    )
  }

  return (
    <div className="flex flex-col mt-8 gap-3 fade-in">
      {/* Toolbar */}
      <div className="flex flex-col sm:flex-row gap-3 items-start sm:items-center justify-between">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
          <input
            value={search}
            onChange={e => setSearch(e.target.value)}
            placeholder="Proje veya yazar ara..."
            className="pl-9 pr-3 h-9 w-64 bg-card border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors placeholder:text-muted-foreground"
          />
        </div>
        <div className="flex items-center gap-2">
          <select
            value={categoryFilter}
            onChange={e => setCategoryFilter(e.target.value)}
            className="h-9 px-3 bg-card border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50"
          >
            {categories.map(c => (
              <option key={c} value={c}>{c}</option>
            ))}
          </select>
          {(search || categoryFilter !== "Tümü") && (
            <button
              onClick={() => {
                setSearch("");
                setCategoryFilter("Tümü");
                if (typeof window !== "undefined") {
                  window.history.pushState({}, "", "/dashboard");
                }
              }}
              className="h-9 px-3 text-xs bg-muted text-muted-foreground border border-border rounded-lg hover:bg-muted/80 transition-colors"
            >
              Sıfırla
            </button>
          )}
        </div>

        {selectedIds.size > 0 && (
          <div className="flex items-center gap-2">
            <span className="text-xs text-muted-foreground">{selectedIds.size} seçili</span>
            {selectedIds.size === 1 && (
              <a
                href={`/project?id=${Array.from(selectedIds)[0]}`}
                className="flex items-center gap-1.5 h-8 px-3 text-xs bg-primary/10 text-primary border border-primary/20 rounded-lg hover:bg-primary/15 transition-colors font-medium"
              >
                <FileText className="w-3.5 h-3.5" /> İncele & PDF
              </a>
            )}
            <button
              onClick={exportXLSX}
              className="flex items-center gap-1.5 h-8 px-3 text-xs bg-card border border-border rounded-lg hover:bg-muted transition-colors"
            >
              <FileOutput className="w-3.5 h-3.5" /> Rapor (XLSX)
            </button>
            <button
              onClick={handleDeleteSelected}
              className="h-8 px-3 text-xs bg-red-500/10 text-red-500 border border-red-500/20 rounded-lg hover:bg-red-500/15 transition-colors"
            >
              Sil
            </button>
          </div>
        )}

        {selectedIds.size === 0 && (
          <button
            onClick={exportXLSX}
            className="h-8 px-3 text-xs bg-card border border-border rounded-lg hover:bg-muted transition-colors flex items-center gap-1.5 text-muted-foreground"
          >
            <Download className="w-3.5 h-3.5" /> XLSX Dışa Aktar
          </button>
        )}
      </div>

      {/* Tablo */}
      <div className="bg-card rounded-xl border border-border overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full text-sm text-left">
            <thead className="border-b border-border bg-muted/30">
              <tr>
                <th className="px-4 py-3 w-10">
                  <input
                    type="checkbox"
                    className="accent-primary rounded"
                    checked={selectedIds.size === filtered.length && filtered.length > 0}
                    onChange={toggleSelectAll}
                  />
                </th>
                <th
                  className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider cursor-pointer hover:text-foreground transition-colors"
                  onClick={() => handleSort("title")}
                >
                  <div className="flex items-center gap-1">Proje Adı <SortIcon col="title" /></div>
                </th>
                <th className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">
                  Durum
                </th>
                <th className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider text-right">
                  Gönderen
                </th>
                <th
                  className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider text-right cursor-pointer hover:text-foreground transition-colors"
                  onClick={() => handleSort("word_count")}
                >
                  <div className="flex items-center justify-end gap-1">Analiz Edilen Kelime <SortIcon col="word_count" /></div>
                </th>
                <th
                  className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider text-right cursor-pointer hover:text-foreground transition-colors"
                  onClick={() => handleSort("score")}
                >
                  <div className="flex items-center justify-end gap-1">Puan <SortIcon col="score" /></div>
                </th>
                <th className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider text-right">
                  Kategori
                </th>
                <th className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider text-right">
                  Detay
                </th>
                <th className="px-4 py-3 text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">
                  Not
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border">
              {filtered.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-10 text-center text-sm text-muted-foreground">
                    {search ? `"${search}" için sonuç bulunamadı` : "Proje bulunamadı"}
                  </td>
                </tr>
              ) : filtered.map((project, index) => (
                 <tr
                  key={project.id}
                  draggable={draggableId === project.id}
                  onDragStart={(e) => handleDragStart(e, project.id)}
                  onDragEnter={(e) => handleDragEnter(e, project.id)}
                  onDragOver={handleDragOver}
                  onDrop={(e) => handleDrop(e, project.id)}
                  onDragEnd={() => { handleDragEnd(); setDraggableId(null) }}
                  className={`hover:bg-muted/30 transition-colors 
                    ${selectedIds.has(project.id) ? "bg-primary/5" : ""}
                    ${draggedItemId === project.id ? "opacity-40 scale-[0.99]" : ""}
                    ${dragOverItemId === project.id && dragOverItemId !== draggedItemId
                      ? (customOrder.indexOf(draggedItemId!) < customOrder.indexOf(project.id) 
                          ? "border-b-2 border-primary shadow-[0_2px_0_hsl(var(--primary))]" 
                          : "border-t-2 border-primary shadow-[0_-2px_0_hsl(var(--primary))]") 
                      : ""}
                  `}
                >
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <div 
                        className="cursor-grab active:cursor-grabbing text-muted-foreground/40 hover:text-primary p-1 rounded hover:bg-primary/10 transition-colors select-none"
                        onMouseDown={() => setDraggableId(project.id)}
                        onMouseUp={() => setDraggableId(null)}
                        title="Sürükleyerek sırala"
                      >
                        <GripVertical className="w-3.5 h-3.5" />
                      </div>
                      <input
                        type="checkbox"
                        className="accent-primary rounded"
                        checked={selectedIds.has(project.id)}
                        onChange={() => toggleSelect(project.id)}
                      />
                    </div>
                  </td>
                  <td className="px-4 py-3 font-medium truncate max-w-[220px]">{project.title}</td>
                  <td className="px-4 py-3">
                    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-semibold uppercase tracking-wide border ${
                      project.score !== null && project.score !== undefined
                        ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20"
                        : project.status === "İnceleniyor"
                        ? "bg-blue-500/10 text-blue-400 border-blue-500/20"
                        : "border-border text-muted-foreground"
                    }`}>
                      {project.score !== null && project.score !== undefined ? "İncelendi"
                        : project.status === "İnceleniyor" ? "İnceleniyor"
                        : "Bekliyor"}
                    </span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="text-xs text-right text-muted-foreground">{project.author || "Bilinmiyor"}</div>
                  </td>
                  <td className="px-4 py-3">
                    <div className="text-sm font-medium text-right">{(project.word_count || 0).toLocaleString("tr-TR")}</div>
                  </td>
                  <td className="px-4 py-3 text-right">
                    <div className="text-sm font-bold">
                      {project.score !== null && project.score !== undefined ? project.score : <span className="text-muted-foreground">-</span>}
                    </div>
                  </td>
                  <td className="px-4 py-3 text-right text-muted-foreground text-xs">{project.category}</td>
                  <td className="px-4 py-3 text-right">
                    <a
                      href={`/project?id=${project.id}`}
                      className="text-xs text-muted-foreground hover:text-foreground bg-muted/40 hover:bg-muted border border-border px-2.5 py-1 rounded-md transition-colors"
                    >
                      Detay
                    </a>
                  </td>
                  <td className="px-4 py-3">
                    <span className={`text-sm font-bold ${gradeClass(project.grade)}`}>
                      {project.grade}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <p className="text-xs text-muted-foreground">
        {filtered.length} / {projects.length} proje
        {search && ` — "${search}" araması`}
      </p>
    </div>
  )
}
