import React, { useState, useEffect } from "react"
import { Plus, Trash2, Save, BrainCircuit, Info } from "lucide-react"

interface Category {
  id: string
  name: string
  prompt: string
}

const DEFAULT_CATEGORIES: Category[] = [
  { id: "1", name: "Genel", prompt: "Projenin genel teknolojik uygulanabilirliğini, toplumsal faydasını, sürdürülebilirliğini ve teknik altyapısını değerlendir." },
  { id: "2", name: "İnsanlık Yararına Teknoloji", prompt: "Projenin dezavantajlı gruplara, toplumsal problemlere veya günlük yaşantıyı kolaylaştırmaya yönelik sunduğu yenilikçi çözümleri ve ölçeklenebilirliğini değerlendir." },
  { id: "3", name: "Eğitim Teknolojileri", prompt: "Projenin öğrenme süreçlerine katkısını, dijitalleşme vizyonunu, eğitime erişimde fırsat eşitliği sağlama potansiyelini ve eğitimdeki teknolojik yenilikleri değerlendir." },
  { id: "4", name: "Çevre ve Enerji Teknolojileri", prompt: "Projenin karbon ayak izini azaltma, yenilenebilir enerji kaynaklarını kullanma, atık yönetimi veya çevre bilinci oluşturma konularındaki başarısını ve uygulanabilirliğini incele." },
  { id: "5", name: "Sağlık Teknolojileri", prompt: "Projenin erken teşhis, tedavi süreçlerini iyileştirme, hasta takibi veya sağlık sektöründeki cihaz gelişimine yaptığı katkıları, teknik güvenilirliğini ve klinik uygulanabilirliğini değerlendir." },
  { id: "6", name: "Ulaşım ve Mobilite Teknolojileri", prompt: "Projenin güvenli, çevre dostu, otonom veya entegre ulaşım sistemleri alanındaki yeniliklerini, trafik yönetimine ve enerji verimliliğine katkısını incele." },
  { id: "7", name: "Tarım Teknolojileri", prompt: "Projenin akıllı tarım uygulamaları, verimlilik artışı, su tasarrufu, hastalık tespiti veya gıda güvenliği konularındaki teknolojik çözümlerini değerlendir." },
  { id: "8", name: "Afet Yönetim Teknolojileri", prompt: "Projenin afet öncesi uyarı, afet anı müdahale ve afet sonrası kurtarma veya koordinasyon süreçlerindeki teknolojik altyapısını, güvenilirliğini ve hızlı müdahale kapasitesini değerlendir." }
]

export default function AiCategoriesPage() {
  const [categories, setCategories] = useState<Category[]>([])
  const [newCatName, setNewCatName] = useState("")
  const [newCatPrompt, setNewCatPrompt] = useState("")
  const [saving, setSaving] = useState(false)

  // Load from local storage
  useEffect(() => {
    const saved = localStorage.getItem("kys_ai_categories_v2")
    if (saved) {
      setCategories(JSON.parse(saved))
    } else {
      setCategories(DEFAULT_CATEGORIES)
      localStorage.setItem("kys_ai_categories_v2", JSON.stringify(DEFAULT_CATEGORIES))
    }
  }, [])

  const handleSave = () => {
    setSaving(true)
    localStorage.setItem("kys_ai_categories_v2", JSON.stringify(categories))
    // Tauri aracılığıyla backend'e gönderilecekse burada tauriInvoke tetiklenebilir
    setTimeout(() => setSaving(false), 500)
  }

  const handleAddCategory = (e: React.FormEvent) => {
    e.preventDefault()
    if (!newCatName.trim()) return

    const newCategory: Category = {
      id: Date.now().toString(),
      name: newCatName.trim(),
      prompt: newCatPrompt.trim() || `${newCatName} alanında yenilikçi yaklaşımları değerlendir.`,
    }

    setCategories([...categories, newCategory])
    setNewCatName("")
    setNewCatPrompt("")
  }

  const handleDelete = (id: string) => {
    setCategories(categories.filter(c => c.id !== id))
  }

  const handlePromptChange = (id: string, newPrompt: string) => {
    setCategories(categories.map(c => c.id === id ? { ...c, prompt: newPrompt } : c))
  }

  return (
    <div className="flex-1 flex flex-col min-w-0 overflow-hidden bg-background text-foreground font-sans antialiased">
      {/* Üst Header */}
      <header className="h-20 border-b border-border bg-card/50 flex items-center px-6 justify-between shrink-0">
        <div className="font-semibold text-lg text-muted-foreground flex items-center gap-2">
          <span className="text-foreground">JanissaryAsistan</span> / Kategori & AI Yönetimi
        </div>
      </header>

      {/* İçerik */}
      <main className="flex-1 overflow-y-auto p-6 md:p-8 custom-scrollbar">
        <div className="max-w-5xl mx-auto space-y-8">
          <div>
            <h1 className="text-3xl font-bold tracking-tight mb-2">AI Değerlendirme Kategorileri</h1>
            <p className="text-muted-foreground">
              Sisteme yüklenecek projelerin hangi kategorilere göre sınıflandırılacağını ve yapay zekanın 
              bu projeleri incelerken hangi kriterleri (prompt) baz alacağını buradan ayarlayabilirsiniz.
            </p>
          </div>

          <div className="bg-primary/5 border border-primary/20 rounded-xl p-4 flex gap-4 text-sm text-primary/90">
            <Info className="w-5 h-5 shrink-0 mt-0.5" />
            <p>
              <strong>İpucu:</strong> Yapay zeka motoru, PDF yüklediğinizde bu kategorileri inceler ve 
              projeyi en uygun olanına atar. Prompt (komut) kutusuna girdiğiniz talimatlar, 
              Teknik Derinlik ve Alan Uyumu puanları verilirken birebir uygulanır.
            </p>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            {/* Sol Taraf: Yeni Kategori Ekleme */}
            <div className="lg:col-span-1">
              <div className="bg-card border border-border rounded-xl shadow-sm p-5">
                <h3 className="font-semibold mb-4 flex items-center gap-2">
                  <Plus className="w-4 h-4 text-primary" /> Yeni Kategori Ekle
                </h3>
                <form onSubmit={handleAddCategory} className="space-y-4">
                  <div>
                    <label className="block text-xs font-medium text-muted-foreground mb-1.5">Kategori Adı</label>
                    <input
                      type="text"
                      value={newCatName}
                      onChange={e => setNewCatName(e.target.value)}
                      placeholder="Örn: Biyoteknoloji"
                      className="w-full h-9 px-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors"
                      required
                    />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-muted-foreground mb-1.5">AI Kriteri (Prompt)</label>
                    <textarea
                      value={newCatPrompt}
                      onChange={e => setNewCatPrompt(e.target.value)}
                      placeholder="Yapay zekanın bu kategoriyi değerlendirirken dikkat etmesi gereken özel noktalar..."
                      className="w-full h-24 p-3 bg-background border border-border rounded-lg text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors resize-none custom-scrollbar"
                    />
                  </div>
                  <button
                    type="submit"
                    disabled={!newCatName.trim()}
                    className="w-full h-9 bg-primary hover:bg-primary/90 text-primary-foreground font-medium rounded-lg text-sm transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    Kategori Ekle
                  </button>
                </form>
              </div>
            </div>

            {/* Sağ Taraf: Kategori Listesi */}
            <div className="lg:col-span-2 space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="font-semibold text-lg flex items-center gap-2">
                  <BrainCircuit className="w-5 h-5 text-primary" /> Aktif Kategoriler
                </h3>
                <button
                  onClick={handleSave}
                  className="h-9 px-4 bg-muted hover:bg-muted/80 text-foreground font-medium rounded-lg text-sm transition-colors flex items-center gap-2 border border-border"
                >
                  <Save className="w-4 h-4" />
                  {saving ? "Kaydedildi!" : "Değişiklikleri Kaydet"}
                </button>
              </div>
              
              {categories.length === 0 ? (
                <div className="p-8 text-center border border-dashed border-border rounded-xl text-muted-foreground">
                  Henüz hiç kategori yok. Soldaki formu kullanarak ekleyebilirsiniz.
                </div>
              ) : (
                <div className="space-y-3">
                  {categories.map((cat) => (
                    <div key={cat.id} className="bg-card border border-border rounded-xl p-4 flex gap-4 group transition-colors hover:border-primary/30">
                      <div className="flex-1 space-y-3">
                        <div className="flex items-center justify-between">
                          <h4 className="font-semibold text-primary">{cat.name}</h4>
                          <button
                            onClick={() => handleDelete(cat.id)}
                            className="text-muted-foreground hover:text-red-500 transition-colors p-1"
                            title="Kategoriyi Sil"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                        <div>
                          <label className="block text-[11px] uppercase tracking-wider font-semibold text-muted-foreground mb-1">
                            AI İnceleme Kriteri
                          </label>
                          <textarea
                            value={cat.prompt}
                            onChange={(e) => handlePromptChange(cat.id, e.target.value)}
                            className="w-full min-h-[60px] p-2 bg-background/50 border border-transparent hover:border-border focus:border-primary/50 rounded-lg text-sm focus:outline-none transition-colors resize-y custom-scrollbar text-muted-foreground focus:text-foreground"
                          />
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </main>
    </div>
  )
}
