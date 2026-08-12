import { useEffect, useState } from "react"
import { supabase } from "@/lib/supabase"
import {
  User, Download, Upload, Moon, Sun, Shield,
  Bell, Key, Trash2, CheckCircle2, AlertCircle,
  FileSpreadsheet, Save, Target
} from "lucide-react"
import * as XLSX from "xlsx"

type Section = "hesap" | "veri" | "kategoriler" | "gorunum" | "guvenlik"

// ─── Küçük UI yardımcıları ────────────────────────────────────────

function SectionCard({ title, icon: Icon, children }: { title: string; icon: any; children: React.ReactNode }) {
  return (
    <div className="bg-card border border-border rounded-xl overflow-hidden">
      <div className="flex items-center gap-3 px-5 py-4 border-b border-border">
        <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
          <Icon className="w-4 h-4 text-primary" />
        </div>
        <h2 className="font-semibold text-sm">{title}</h2>
      </div>
      <div className="p-5 space-y-4">{children}</div>
    </div>
  )
}

function FieldRow({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
      <div className="sm:w-48 shrink-0">
        <p className="text-sm font-medium">{label}</p>
        {hint && <p className="text-xs text-muted-foreground mt-0.5">{hint}</p>}
      </div>
      <div className="flex-1">{children}</div>
    </div>
  )
}

function TextInput({ value, onChange, placeholder, type = "text", disabled = false }: any) {
  return (
    <input
      type={type}
      value={value}
      onChange={e => onChange(e.target.value)}
      placeholder={placeholder}
      disabled={disabled}
      className="w-full h-9 px-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors disabled:opacity-50 disabled:cursor-not-allowed placeholder:text-muted-foreground"
    />
  )
}

function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (v: boolean) => void; label?: string }) {
  return (
    <label className="flex items-center gap-3 cursor-pointer group">
      <div
        onClick={() => onChange(!checked)}
        className={`relative w-10 h-5 rounded-full transition-colors ${checked ? "bg-primary" : "bg-border"}`}
      >
        <div className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform ${checked ? "translate-x-5" : "translate-x-0"}`} />
      </div>
      {label && <span className="text-sm text-muted-foreground group-hover:text-foreground transition-colors">{label}</span>}
    </label>
  )
}

function Toast({ msg, type }: { msg: string; type: "success" | "error" }) {
  return (
    <div className={`fixed bottom-6 right-6 flex items-center gap-2 px-4 py-3 rounded-xl shadow-lg border text-sm font-medium z-50 fade-in
      ${type === "success" ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20" : "bg-red-500/10 text-red-500 border-red-500/20"}`}>
      {type === "success" ? <CheckCircle2 className="w-4 h-4" /> : <AlertCircle className="w-4 h-4" />}
      {msg}
    </div>
  )
}

// ─── Ana Ayarlar Bileşeni ─────────────────────────────────────────

export function SettingsPage() {
  const [section, setSection] = useState<Section>("hesap")
  const [toast, setToast] = useState<{ msg: string; type: "success" | "error" } | null>(null)

  useEffect(() => {
    if (typeof window !== "undefined") {
      const urlParams = new URLSearchParams(window.location.search)
      const tab = urlParams.get('tab') as Section
      if (tab) setSection(tab)
    }
  }, [])

  // Hesap
  const [userEmail, setUserEmail] = useState("")
  const [fullName, setFullName] = useState("")
  const [userRole, setUserRole] = useState("Jüri Üyesi")
  const [savingProfile, setSavingProfile] = useState(false)

  // Kategoriler
  const [categories, setCategories] = useState<any[]>([])
  const [newCatName, setNewCatName] = useState("")
  const [newCatPrompt, setNewCatPrompt] = useState("")
  const [loadingCats, setLoadingCats] = useState(false)
  
  // API Key State
  const [apiKey, setApiKey] = useState("")
  const [savingApiKey, setSavingApiKey] = useState(false)
  
  const [serperKey, setSerperKey] = useState("")
  const [savingSerperKey, setSavingSerperKey] = useState(false)
  
  const [systemPrompt, setSystemPrompt] = useState("")
  const [savingPrompt, setSavingPrompt] = useState(false)

  // Kategorileri Getir
  useEffect(() => {
    if (section === "kategoriler") {
      const fetchCategories = async () => {
        setLoadingCats(true)
        const { data } = await supabase.from("evaluation_categories").select("*").order("id")
        setCategories(data || [])
        setLoadingCats(false)
      }
      const fetchApiKeys = async () => {
        try {
          const res1 = await fetch("http://localhost:8080/api/settings/openai-key")
          const j1 = await res1.json()
          if (j1.status === "success" && j1.data?.has_key) setApiKey(j1.data.masked_key)

          const res2 = await fetch("http://localhost:8080/api/settings/serper-key")
          const j2 = await res2.json()
          if (j2.status === "success" && j2.data?.has_key) setSerperKey(j2.data.masked_key)
          
          const res3 = await fetch("http://localhost:8080/api/settings/system-prompt")
          const j3 = await res3.json()
          if (j3.status === "success") setSystemPrompt(j3.data?.prompt || "")
        } catch (e) { console.warn("API Key alınamadı:", e) }
      }
      fetchCategories()
      fetchApiKeys()
    }
  }, [section])

  const addCategory = async () => {
    if (!newCatName || !newCatPrompt) return showToast("Tüm alanları doldurun", "error")
    const { error } = await supabase.from("evaluation_categories").insert([{ name: newCatName, criteria_prompt: newCatPrompt }])
    if (error) { showToast("Eklenemedi: " + error.message, "error"); return }
    showToast("Kategori eklendi", "success")
    setNewCatName(""); setNewCatPrompt("")
    // refresh
    const { data } = await supabase.from("evaluation_categories").select("*").order("id")
    setCategories(data || [])
  }

  const deleteCategory = async (id: number) => {
    if (!confirm("Emin misiniz?")) return
    const { error } = await supabase.from("evaluation_categories").delete().eq("id", id)
    if (error) { showToast("Silinemedi", "error"); return }
    showToast("Kategori silindi", "success")
    const { data } = await supabase.from("evaluation_categories").select("*").order("id")
    setCategories(data || [])
  }

  const saveApiKey = async () => {
    if (!apiKey) return showToast("API Key giriniz", "error")
    if (apiKey.includes("...")) return showToast("Lütfen yeni bir API Key girin (örn: sk-...)", "error")
    setSavingApiKey(true)
    try {
      const res = await fetch("http://localhost:8080/api/settings/openai-key", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ api_key: apiKey })
      })
      if (!res.ok) throw new Error("Sunucu hatası")
      showToast("OpenAI Key kaydedildi", "success")
    } catch (e) {
      showToast("Kaydedilemedi, sunucu çalışıyor mu?", "error")
    }
    setSavingApiKey(false)
  }

  const saveSerperKey = async () => {
    if (!serperKey) return showToast("Serper API Key giriniz", "error")
    if (serperKey.includes("...")) return showToast("Lütfen yeni bir Serper Key girin", "error")
    setSavingSerperKey(true)
    try {
      const res = await fetch("http://localhost:8080/api/settings/serper-key", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ api_key: serperKey })
      })
      if (!res.ok) throw new Error("Sunucu hatası")
      showToast("Serper.dev Key kaydedildi", "success")
    } catch (e) {
      showToast("Kaydedilemedi", "error")
    } finally {
      setSavingSerperKey(false)
    }
  }
  
  const saveSystemPrompt = async () => {
    setSavingPrompt(true)
    try {
      const res = await fetch("http://localhost:8080/api/settings/system-prompt", {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ prompt: systemPrompt })
      })
      if (!res.ok) throw new Error("Sunucu hatası")
      showToast("Ana Prompt kaydedildi", "success")
    } catch (e) {
      showToast("Kaydedilemedi", "error")
    }
    setSavingPrompt(false)
  }

  // Görünüm
  const [theme, setTheme] = useState<"dark" | "light">("dark")
  const [compactMode, setCompactMode] = useState(false)
  const [animations, setAnimations] = useState(true)

  // Bildirimler
  const [notifHighRisk, setNotifHighRisk] = useState(true)
  const [notifNewUpload, setNotifNewUpload] = useState(true)
  const [notifComplete, setNotifComplete] = useState(false)

  // Güvenlik
  const [newPass, setNewPass] = useState("")
  const [newPass2, setNewPass2] = useState("")
  const [changingPass, setChangingPass] = useState(false)

  // Başlangıçta kullanıcı verilerini çek
  useEffect(() => {
    supabase.auth.getUser().then(({ data: { user } }) => {
      if (user) {
        setUserEmail(user.email || "")
        setFullName(user.user_metadata?.full_name || "")
      }
    })
    // Tema
    const saved = localStorage.getItem("theme")
    setTheme(saved === "dark" ? "dark" : "light")
  }, [])

  function showToast(msg: string, type: "success" | "error") {
    setToast({ msg, type })
    setTimeout(() => setToast(null), 3500)
  }

  // Tema toggle
  const handleThemeToggle = (dark: boolean) => {
    const t = dark ? "dark" : "light"
    setTheme(t)
    localStorage.setItem("theme", t)
    document.documentElement.classList.toggle("dark", dark)
  }

  // Profil kaydet
  const handleSaveProfile = async () => {
    setSavingProfile(true)
    try {
      const { error } = await supabase.auth.updateUser({ data: { full_name: fullName } })
      if (error) throw error
      showToast("Profil güncellendi", "success")
    } catch (e: any) {
      showToast(e.message || "Bir hata oluştu", "error")
    } finally {
      setSavingProfile(false)
    }
  }



  // XLSX Dışa Aktarım (tüm projeler)
  const exportAll = async () => {
    try {
      const { data, error } = await supabase
        .from("projects")
        .select("id, filename, category, grade, status, created_at")
        .order("id")
      if (error || !data) throw error

      const ws = XLSX.utils.json_to_sheet(data)
      const wb = XLSX.utils.book_new()
      XLSX.utils.book_append_sheet(wb, ws, "Projeler")
      XLSX.writeFile(wb, `janissary-tum-projeler-${new Date().toISOString().slice(0,10)}.xlsx`)
      showToast(`${data.length} proje dışa aktarıldı`, "success")
    } catch {
      showToast("Dışa aktarım başarısız", "error")
    }
  }

  // Şifre değiştir
  const handleChangePassword = async () => {
    if (!newPass || newPass !== newPass2) {
      showToast("Şifreler eşleşmiyor", "error"); return
    }
    if (newPass.length < 6) {
      showToast("Şifre en az 6 karakter olmalı", "error"); return
    }
    setChangingPass(true)
    try {
      const { error } = await supabase.auth.updateUser({ password: newPass })
      if (error) throw error
      setNewPass(""); setNewPass2("")
      showToast("Şifre güncellendi", "success")
    } catch (e: any) {
      showToast(e.message || "Hata", "error")
    } finally {
      setChangingPass(false)
    }
  }

  const navItems: { id: Section; label: string; icon: any }[] = [
    { id: "hesap", label: "Hesap", icon: User },
    { id: "veri", label: "Veri Yönetimi", icon: FileSpreadsheet },
    { id: "kategoriler", label: "Kategoriler & AI", icon: Target },
    { id: "gorunum", label: "Görünüm", icon: Sun },
    { id: "guvenlik", label: "Güvenlik", icon: Shield },
  ]

  return (
    <div className="flex gap-6 h-full fade-in">
      {/* Sol nav */}
      <div className="w-48 shrink-0">
        <nav className="flex flex-col gap-1">
          {navItems.map(item => {
            const Icon = item.icon
            return (
              <button
                key={item.id}
                onClick={() => setSection(item.id)}
                className={`flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors text-left
                  ${section === item.id
                    ? "bg-primary/10 text-primary dark:text-primary"
                    : "text-muted-foreground hover:bg-muted hover:text-foreground"}`}
              >
                <Icon className="w-4 h-4 shrink-0" />
                {item.label}
              </button>
            )
          })}
        </nav>
      </div>

      {/* Sağ içerik */}
      <div className="flex-1 overflow-y-auto custom-scrollbar space-y-4 pb-8">

        {/* ─ Hesap ─ */}
        {section === "hesap" && (
          <SectionCard title="Hesap Bilgileri" icon={User}>
            <FieldRow label="Ad Soyad" hint="Profilinde gösterilir">
              <TextInput value={fullName} onChange={setFullName} placeholder="Adınız Soyadınız" />
            </FieldRow>
            <FieldRow label="E-posta" hint="Değiştirmek için destek al">
              <TextInput value={userEmail} onChange={() => {}} placeholder="—" disabled />
            </FieldRow>
            <FieldRow label="Rol">
              <select
                value={userRole}
                onChange={e => setUserRole(e.target.value)}
                className="h-9 px-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 text-foreground"
              >
                <option>Jüri Üyesi</option>
                <option>Koordinatör</option>
                <option>Gözlemci</option>
                <option>Yönetici</option>
              </select>
            </FieldRow>
            <div className="pt-2 border-t border-border flex justify-end">
              <button
                onClick={handleSaveProfile}
                disabled={savingProfile}
                className="flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
              >
                <Save className="w-4 h-4" />
                {savingProfile ? "Kaydediliyor..." : "Kaydet"}
              </button>
            </div>
          </SectionCard>
        )}



        {/* ─ Veri Yönetimi ─ */}
        {section === "veri" && (
          <SectionCard title="Veri Yönetimi" icon={FileSpreadsheet}>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-4 bg-muted/20 rounded-lg border border-border">
                <div>
                  <p className="text-sm font-medium">Tüm Projeleri Dışa Aktar</p>
                  <p className="text-xs text-muted-foreground mt-0.5">projects tablosundaki tüm kayıtları XLSX olarak indir</p>
                </div>
                <button
                  onClick={exportAll}
                  className="flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
                >
                  <Download className="w-4 h-4" /> XLSX İndir
                </button>
              </div>

              <div className="flex items-center justify-between p-4 bg-muted/20 rounded-lg border border-border">
                <div>
                  <p className="text-sm font-medium">Puanlama Verisi Dışa Aktar</p>
                  <p className="text-xs text-muted-foreground mt-0.5">scores tablosunu detaylı XLSX olarak indir</p>
                </div>
                <button
                  onClick={async () => {
                    const { data } = await supabase.from("scores").select("*").order("id")
                    if (!data?.length) { showToast("Veri bulunamadı", "error"); return }
                    const ws = XLSX.utils.json_to_sheet(data)
                    const wb = XLSX.utils.book_new()
                    XLSX.utils.book_append_sheet(wb, ws, "Puanlar")
                    XLSX.writeFile(wb, `janissary-puanlar-${new Date().toISOString().slice(0,10)}.xlsx`)
                    showToast(`${data.length} puan kaydı dışa aktarıldı`, "success")
                  }}
                  className="flex items-center gap-2 h-9 px-4 bg-card border border-border rounded-lg text-sm hover:bg-muted transition-colors"
                >
                  <Download className="w-4 h-4" /> İndir
                </button>
              </div>

              <div className="flex items-center justify-between p-4 bg-muted/20 rounded-lg border border-border">
                <div>
                  <p className="text-sm font-medium">CSV'den Proje İçe Aktar</p>
                  <p className="text-xs text-muted-foreground mt-0.5">filename, category, grade sütunları olmalı</p>
                </div>
                <label className="flex items-center gap-2 h-9 px-4 bg-card border border-border rounded-lg text-sm hover:bg-muted transition-colors cursor-pointer">
                  <Upload className="w-4 h-4" /> CSV Yükle
                  <input
                    type="file"
                    accept=".csv"
                    className="hidden"
                    onChange={async e => {
                      const file = e.target.files?.[0]; if (!file) return
                      const text = await file.text()
                      const lines = text.split("\n").filter(Boolean)
                      const headers = lines[0].split(",").map(h => h.trim().replace(/"/g, ""))
                      const rows = lines.slice(1).map(l => {
                        const vals = l.split(",").map(v => v.trim().replace(/"/g, ""))
                        const obj: any = {}
                        headers.forEach((h, i) => { obj[h] = vals[i] || "" })
                        return obj
                      })
                      const toInsert = rows
                        .filter(r => r.filename)
                        .map(r => ({ filename: r.filename, category: r.category || "Genel", grade: r.grade || "-", status: "İnceleniyor", file_type: "Pdf" }))

                      if (!toInsert.length) { showToast("Geçerli satır bulunamadı", "error"); return }
                      const { error } = await supabase.from("projects").insert(toInsert)
                      if (error) { showToast("İçe aktarım hatası: " + error.message, "error"); return }
                      showToast(`${toInsert.length} proje içe aktarıldı`, "success")
                      e.target.value = ""
                    }}
                  />
                </label>
              </div>
            </div>
          </SectionCard>
        )}

        {/* ─ Kategoriler ─ */}
        {section === "kategoriler" && (
          <div className="space-y-4">
            <SectionCard title="AI Bağlantı Ayarları" icon={Key}>
              <FieldRow label="OpenAI API Key" hint="Gpt-4o-mini entegrasyonu için">
                <TextInput 
                  value={apiKey} 
                  onChange={setApiKey} 
                  placeholder="sk-proj-..." 
                  type={apiKey.includes("...") ? "text" : "password"}
                />
              </FieldRow>
              <div className="pt-2 border-t border-border flex justify-end">
                <button
                  onClick={saveApiKey}
                  disabled={savingApiKey}
                  className="flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 disabled:opacity-50 transition-colors"
                >
                  <Save className="w-4 h-4" />
                  {savingApiKey ? "Kaydediliyor..." : "Anahtarı Kaydet"}
                </button>
              </div>
            </SectionCard>

            <SectionCard title="Serper.dev API" icon={Key}>
              <p className="text-sm text-muted-foreground mb-4">Web araştırması ve dosya analizi için Serper.dev API anahtarı.</p>
              <div className="flex gap-2">
                <TextInput value={serperKey} onChange={setSerperKey} placeholder="9e7b..." type={serperKey.includes("...") ? "text" : "password"} />
                <button onClick={saveSerperKey} disabled={savingSerperKey} className="bg-primary text-primary-foreground px-4 py-2 rounded-lg font-medium whitespace-nowrap disabled:opacity-50">
                  {savingSerperKey ? "Kaydediliyor..." : "Kaydet"}
                </button>
              </div>
            </SectionCard>
            
            <SectionCard title="Sistem Ana Prompt (Genel Yönerge)" icon={FileSpreadsheet}>
              <p className="text-sm text-muted-foreground mb-4">Yapay zeka asistanına gönderilen varsayılan genel analiz komutunu (prompt) buradan belirleyebilirsiniz.</p>
              <div className="flex flex-col gap-3 mt-4">
                <textarea 
                  value={systemPrompt} 
                  onChange={e => setSystemPrompt(e.target.value)} 
                  placeholder="Projenin bilimsel derinliğini ve yapay zeka tarafından yazılma ihtimalini değerlendir..."
                  className="w-full min-h-[120px] p-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:border-primary/50"
                />
                <button onClick={saveSystemPrompt} disabled={savingPrompt} className="bg-primary text-primary-foreground px-4 py-2 rounded-lg font-medium self-end disabled:opacity-50">
                  {savingPrompt ? "Kaydediliyor..." : "Ana Promptu Kaydet"}
                </button>
              </div>
            </SectionCard>

            <SectionCard title="Değerlendirme Kategorileri" icon={Target}>
              <div className="mb-4">
              <p className="text-sm text-muted-foreground">Sisteme yüklenen projeler, burada belirleyeceğin kategorilere göre yapay zeka tarafından puanlanacaktır. Kategori kurallarını olabildiğince detaylı ve anlaşılır yaz.</p>
            </div>
            
            <div className="space-y-4 mb-6">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="md:col-span-1">
                  <p className="text-sm font-medium mb-1">Kategori Adı</p>
                  <TextInput value={newCatName} onChange={setNewCatName} placeholder="Örn: Yapay Zeka, Web Tasarım" />
                </div>
                <div className="md:col-span-2">
                  <p className="text-sm font-medium mb-1">Kurallar & Yönergeler (Prompt)</p>
                  <textarea
                    value={newCatPrompt}
                    onChange={e => setNewCatPrompt(e.target.value)}
                    placeholder="Bu kategorideki projelerin nasıl değerlendirilmesi gerektiğini açıklayın..."
                    className="w-full h-24 p-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors placeholder:text-muted-foreground resize-none"
                  />
                </div>
              </div>
              <div className="flex justify-end">
                <button
                  onClick={addCategory}
                  className="flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
                >
                  <Save className="w-4 h-4" /> Yeni Kategori Ekle
                </button>
              </div>
            </div>

            <div className="space-y-3">
              <p className="font-semibold text-sm">Mevcut Kategoriler</p>
              {loadingCats ? (
                <p className="text-xs text-muted-foreground">Yükleniyor...</p>
              ) : categories.length === 0 ? (
                <p className="text-xs text-muted-foreground">Henüz kategori bulunmuyor.</p>
              ) : (
                categories.map(c => (
                  <div key={c.id} className="p-4 bg-muted/20 border border-border rounded-lg relative group">
                    <button 
                      onClick={() => deleteCategory(c.id)}
                      className="absolute top-2 right-2 p-1.5 text-muted-foreground hover:text-red-500 hover:bg-red-500/10 rounded-md transition-colors opacity-0 group-hover:opacity-100"
                      title="Sil"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                    <h3 className="font-semibold text-primary">{c.name}</h3>
                    <p className="text-sm text-muted-foreground mt-2 break-words">{c.criteria_prompt}</p>
                  </div>
                ))
              )}
            </div>
          </SectionCard>
          </div>
        )}

        {/* ─ Görünüm ─ */}
        {section === "gorunum" && (
          <SectionCard title="Görünüm Ayarları" icon={theme === "dark" ? Moon : Sun}>
            <FieldRow label="Karanlık Mod" hint="Göz dostu koyu tema">
              <Toggle
                checked={theme === "dark"}
                onChange={v => handleThemeToggle(v)}
                label={theme === "dark" ? "Aktif" : "Kapalı"}
              />
            </FieldRow>
            <FieldRow label="Kompakt Görünüm" hint="Daha sık satır aralığı">
              <Toggle
                checked={compactMode}
                onChange={setCompactMode}
                label={compactMode ? "Aktif" : "Kapalı"}
              />
            </FieldRow>
            <FieldRow label="Animasyonlar" hint="Sayfa geçiş efektleri">
              <Toggle
                checked={animations}
                onChange={setAnimations}
                label={animations ? "Aktif" : "Kapalı"}
              />
            </FieldRow>

            <div className="pt-3 border-t border-border">
              <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">Önizleme</p>
              <div className={`rounded-xl border border-border overflow-hidden text-sm ${theme === "dark" ? "bg-[hsl(224,22%,10%)] text-[hsl(220,15%,86%)]" : "bg-white text-gray-900"}`}>
                <div className={`px-4 py-3 border-b flex justify-between items-center ${theme === "dark" ? "border-[hsl(224,18%,16%)] bg-[hsl(224,25%,7%)]" : "border-gray-200 bg-gray-50"}`}>
                  <span className="font-medium">JanissaryAsistan Dashboard</span>
                  <div className="flex gap-1.5">
                    <div className="w-2.5 h-2.5 rounded-full bg-red-400" />
                    <div className="w-2.5 h-2.5 rounded-full bg-yellow-400" />
                    <div className="w-2.5 h-2.5 rounded-full bg-green-400" />
                  </div>
                </div>
                <div className="p-4 grid grid-cols-3 gap-3">
                  {["92", "85", "72"].map((v, i) => (
                    <div key={i} className={`rounded-lg p-3 border ${theme === "dark" ? "bg-[hsl(224,25%,7%)] border-[hsl(224,18%,16%)]" : "bg-gray-50 border-gray-200"}`}>
                      <p className={`text-xs ${theme === "dark" ? "text-[hsl(220,10%,50%)]" : "text-gray-500"}`}>Proje {i+1}</p>
                      <p className="text-xl font-bold mt-1">{v}</p>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </SectionCard>
        )}

        {/* ─ Güvenlik ─ */}
        {section === "guvenlik" && (
          <>
            <SectionCard title="Şifre Değiştir" icon={Key}>
              <FieldRow label="Yeni Şifre">
                <TextInput type="password" value={newPass} onChange={setNewPass} placeholder="••••••••" />
              </FieldRow>
              <FieldRow label="Şifre Tekrar">
                <TextInput type="password" value={newPass2} onChange={setNewPass2} placeholder="••••••••" />
              </FieldRow>
              {newPass && newPass !== newPass2 && (
                <p className="text-xs text-red-500">Şifreler eşleşmiyor</p>
              )}
              <div className="pt-2 border-t border-border flex justify-end">
                <button
                  onClick={handleChangePassword}
                  disabled={changingPass || !newPass || newPass !== newPass2}
                  className="flex items-center gap-2 h-9 px-4 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 disabled:opacity-40 transition-colors"
                >
                  <Key className="w-4 h-4" />
                  {changingPass ? "Güncelleniyor..." : "Şifreyi Güncelle"}
                </button>
              </div>
            </SectionCard>

            <SectionCard title="Bildirimler" icon={Bell}>
              <FieldRow label="Yüksek Risk Uyarısı" hint="Kopya tespitinde bildirim">
                <Toggle checked={notifHighRisk} onChange={setNotifHighRisk} label={notifHighRisk ? "Açık" : "Kapalı"} />
              </FieldRow>
              <FieldRow label="Yeni Proje Yüklendi" hint="PDF yüklendiğinde">
                <Toggle checked={notifNewUpload} onChange={setNotifNewUpload} label={notifNewUpload ? "Açık" : "Kapalı"} />
              </FieldRow>
              <FieldRow label="Analiz Tamamlandı" hint="Puanlama bittiğinde">
                <Toggle checked={notifComplete} onChange={setNotifComplete} label={notifComplete ? "Açık" : "Kapalı"} />
              </FieldRow>
            </SectionCard>

            <SectionCard title="Tehlikeli Alan" icon={Trash2}>
              <div className="flex items-center justify-between p-4 border border-red-500/20 bg-red-500/5 rounded-lg">
                <div>
                  <p className="text-sm font-medium text-red-500">Hesabı Sil</p>
                  <p className="text-xs text-muted-foreground mt-0.5">Bu işlem geri alınamaz. Tüm veriler silinir.</p>
                </div>
                <button
                  onClick={() => alert("Bu özellik yönetici onayı gerektiriyor.")}
                  className="h-9 px-4 bg-red-500/10 text-red-500 border border-red-500/20 rounded-lg text-sm hover:bg-red-500/20 transition-colors"
                >
                  Hesabı Sil
                </button>
              </div>
            </SectionCard>
          </>
        )}

      </div>

      {/* Toast */}
      {toast && <Toast msg={toast.msg} type={toast.type} />}
    </div>
  )
}
