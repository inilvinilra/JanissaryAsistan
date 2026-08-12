import React, { useState, useRef } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { FileText, Loader2, CheckCircle2, AlertCircle, X, UploadCloud } from "lucide-react"

type StagedFile = {
  id: string;
  path: string;
  originalName: string;
  title: string;
  category: string;
  file?: File;
}

type QueueItem = {
  id: string;
  name: string;
  status: "pending" | "processing" | "done" | "error";
  resultId?: string;
  progress?: number;
}

const CATEGORIES = [
  "Genel",
  "İnsanlık Yararına Teknoloji",
  "Eğitim Teknolojileri",
  "Çevre ve Enerji Teknolojileri",
  "Sağlık Teknolojileri",
  "Ulaşım ve Mobilite Teknolojileri",
  "Tarım Teknolojileri",
  "Afet Yönetim Teknolojileri"
];

export function FileUploader() {
  const [stagedFiles, setStagedFiles] = useState<StagedFile[]>([])
  const [queue, setQueue] = useState<QueueItem[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const processFile = async (staged: StagedFile) => {
    const id = staged.id
    setQueue(prev => [...prev, { id, name: staged.title, status: "processing", progress: 0 }])
    
    try {
      if (staged.file) {
        // Tarayıcıdan seçilmiş gerçek dosya varsa API'ye gönder
        const formData = new FormData()
        formData.append("file", staged.file)
        formData.append("author", "Web Kullanıcısı")
        // Backend şu an category ve title almıyor olabilir ama biz gönderelim
        formData.append("title", staged.title)
        
        const res = await fetch("http://localhost:8080/api/analyze", {
          method: "POST",
          body: formData
        })
        
        if (!res.ok) throw new Error("Yükleme başarısız")
        const json = await res.json()
        
        if (json.status === "error") {
            throw new Error(json.message || "Yükleme başarısız")
        }
        
        setQueue(prev => prev.map(q => q.id === id ? { ...q, status: "done", resultId: json.data?.id, progress: 100 } : q))
      } else {
        // Tauri dialog fallback veya test
        await tauriInvoke("upload_project_only", { 
          filePath: staged.path,
          title: staged.title,
          category: staged.category
        })
        setQueue(prev => prev.map(q => q.id === id ? { ...q, status: "done", progress: 100 } : q))
      }
    } catch (error) {
      setQueue(prev => prev.map(q => q.id === id ? { ...q, status: "error" } : q))
    }
  }

  const handleUploadAll = async () => {
    if (stagedFiles.length === 0) return;
    
    // İşlem kuyruğunu hazırla ve dosyaları teker teker yükle
    const filesToUpload = [...stagedFiles];
    setStagedFiles([]); // Staged listesini temizle
    
    for (const file of filesToUpload) {
      await processFile(file);
    }
    
    // Yüklemeler bittikten 1.5 sn sonra Yüklenenler sayfasına git
    setTimeout(() => {
      window.location.href = "/uploads"
    }, 1500);
  }

  const handleSelectFiles = async () => {
    try {
      if (typeof window !== "undefined" && (window as any).__TAURI__) {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const selected = await open({
          multiple: true,
          filters: [{ name: 'PDF', extensions: ['pdf'] }]
        });
        
        if (selected) {
          const files = Array.isArray(selected) ? selected : [selected];
          const newStaged = files.map(path => {
            const name = path.split('\\').pop()?.split('/').pop() || "Bilinmeyen PDF";
            const title = name.replace(".pdf", "");
            return {
              id: Math.random().toString(36).substr(2, 9),
              path,
              originalName: name,
              title,
              category: "İnsanlık Yararına Teknoloji" // Default
            }
          });
          setStagedFiles(prev => [...prev, ...newStaged]);
        }
      } else {
        fileInputRef.current?.click();
      }
    } catch (e) {
      console.warn("Tauri dialog failed, using fallback.", e);
      fileInputRef.current?.click();
    }
  }

  const handleFallbackChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      const newStaged = Array.from(e.target.files).map(file => {
        const title = file.name.replace(".pdf", "");
        return {
          id: Math.random().toString(36).substr(2, 9),
          path: file.name, // Tarayıcıda tam yol alınamaz, sadece isim
          originalName: file.name,
          title,
          category: "Belirtilmedi",
          file // GERÇEK DOSYAYI KAYDET
        }
      });
      setStagedFiles(prev => [...prev, ...newStaged]);
      e.target.value = '';
    }
  }

  const removeStaged = (id: string) => {
    setStagedFiles(prev => prev.filter(f => f.id !== id));
  }

  const updateStaged = (id: string, field: "title" | "category", value: string) => {
    setStagedFiles(prev => prev.map(f => f.id === id ? { ...f, [field]: value } : f));
  }

  return (
    <div className="flex flex-col gap-8">
      <input 
        type="file" 
        multiple 
        accept=".pdf" 
        className="hidden" 
        ref={fileInputRef} 
        onChange={handleFallbackChange} 
      />

      {/* Sürükle Bırak Alanı */}
      <div 
        onClick={handleSelectFiles}
        onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={(e) => {
          e.preventDefault();
          setIsDragging(false);
          if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            const newStaged = Array.from(e.dataTransfer.files).map(file => ({
              id: Math.random().toString(36).substr(2, 9),
              path: file.name,
              originalName: file.name,
              title: file.name.replace(".pdf", ""),
              category: "Belirtilmedi",
              file // GERÇEK DOSYAYI KAYDET
            }));
            setStagedFiles(prev => [...prev, ...newStaged]);
          }
        }}
        className={`bg-card rounded-2xl border-2 border-dashed p-10 lg:p-14 flex flex-col items-center justify-center text-center transition-all cursor-pointer
          ${isDragging ? 'border-primary bg-primary/10 scale-[1.02]' : 'border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/10'}`}
      >
        <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mb-6">
          <UploadCloud className="w-8 h-8 text-primary" />
        </div>
        
        <h3 className="text-xl font-semibold mb-2">PDF Dosyalarını Sürükleyin</h3>
        <p className="text-muted-foreground mb-6 max-w-sm">
          veya bilgisayarınızdan toplu seçmek için tıklayın. Sadece .pdf formatı.
        </p>
        <button className="bg-primary text-primary-foreground hover:bg-primary/90 px-6 py-2.5 rounded-lg font-medium transition-colors pointer-events-none">
          Dosya Seç
        </button>
      </div>

      {/* Hazırlık (Staging) Listesi */}
      {stagedFiles.length > 0 && (
        <div className="bg-card border border-border rounded-xl shadow-sm overflow-hidden fade-in">
          <div className="bg-muted/30 px-5 py-4 border-b border-border flex items-center justify-between">
            <div>
              <h4 className="font-semibold">Seçilen Dosyalar</h4>
              <p className="text-xs text-muted-foreground mt-1">Yüklemeden önce başlık ve kategori belirleyebilirsiniz.</p>
            </div>
            <button 
              onClick={handleUploadAll}
              className="bg-primary text-primary-foreground px-4 py-2 rounded-lg text-sm font-semibold hover:bg-primary/90 transition-colors"
            >
              Tümünü Yükle ({stagedFiles.length})
            </button>
          </div>
          <ul className="divide-y divide-border">
            {stagedFiles.map((file) => (
              <li key={file.id} className="p-4 flex flex-col sm:flex-row sm:items-center gap-4 hover:bg-muted/5 transition-colors">
                <div className="p-3 rounded-lg bg-primary/10 text-primary shrink-0 self-start sm:self-center">
                  <FileText className="w-5 h-5" />
                </div>
                
                <div className="flex-1 grid grid-cols-1 sm:grid-cols-2 gap-4 w-full">
                  <div>
                    <label className="text-xs font-semibold text-muted-foreground mb-1.5 block">Proje Başlığı</label>
                    <input 
                      type="text" 
                      value={file.title} 
                      onChange={(e) => updateStaged(file.id, "title", e.target.value)}
                      className="w-full bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:outline-none focus:border-primary/50"
                      placeholder="Başlık girin..."
                    />
                  </div>
                  <div>
                    <label className="text-xs font-semibold text-muted-foreground mb-1.5 block">Kategori</label>
                    <select 
                      value={file.category}
                      onChange={(e) => updateStaged(file.id, "category", e.target.value)}
                      className="w-full bg-background border border-border rounded-md px-3 py-1.5 text-sm focus:outline-none focus:border-primary/50"
                    >
                      <option value="">Belirtilmedi</option>
                      {CATEGORIES.map(c => (
                        <option key={c} value={c}>{c}</option>
                      ))}
                    </select>
                  </div>
                </div>

                <button 
                  onClick={() => removeStaged(file.id)}
                  className="shrink-0 p-2 text-muted-foreground hover:text-red-500 hover:bg-red-500/10 rounded-lg transition-colors self-end sm:self-center"
                  title="Listeden Çıkar"
                >
                  <X className="w-4 h-4" />
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Kuyruk (Queue) Listesi */}
      {queue.length > 0 && (
        <div className="bg-card border border-border rounded-xl shadow-sm overflow-hidden fade-in">
          <div className="bg-muted/30 px-5 py-3 border-b border-border flex items-center justify-between">
            <h4 className="font-semibold text-sm">Yükleme Durumu</h4>
            <span className="text-xs bg-primary/20 text-primary px-2 py-1 rounded-md font-medium">
              {queue.filter(q => q.status === "processing").length} İşleniyor
            </span>
          </div>
          <ul className="divide-y divide-border max-h-[300px] overflow-y-auto custom-scrollbar">
            {queue.map(item => (
              <li key={item.id} className="p-4 flex items-center justify-between hover:bg-muted/10 transition-colors">
                <div className="flex items-center gap-4">
                  <div className={`p-2 rounded-lg ${
                    item.status === 'processing' ? 'bg-blue-500/10 text-blue-500' :
                    item.status === 'done' ? 'bg-green-500/10 text-green-500' : 'bg-red-500/10 text-red-500'
                  }`}>
                    <FileText className="w-5 h-5" />
                  </div>
                  <div>
                    <p className="text-sm font-medium">{item.name}</p>
                    <div className="flex items-center gap-2 mt-1">
                      {item.status === 'processing' && <Loader2 className="w-3 h-3 animate-spin text-blue-500" />}
                      {item.status === 'done' && <CheckCircle2 className="w-3 h-3 text-green-500" />}
                      {item.status === 'error' && <AlertCircle className="w-3 h-3 text-red-500" />}
                      <span className="text-xs text-muted-foreground">
                        {item.status === 'processing' ? 'Veritabanına Yükleniyor...' : 
                         item.status === 'done' ? 'Yükleme Tamamlandı' : 'Hata Oluştu'}
                      </span>
                    </div>
                  </div>
                </div>
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}
