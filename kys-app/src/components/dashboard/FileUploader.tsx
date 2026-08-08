import React, { useState, useRef } from "react"
import { tauriInvoke } from "@/lib/tauri"
import { FileText, Loader2, CheckCircle2, AlertCircle } from "lucide-react"

type QueueItem = {
  id: string;
  name: string;
  status: "pending" | "processing" | "done" | "error";
  resultId?: string;
  progress?: number;
}

export function FileUploader() {
  const [queue, setQueue] = useState<QueueItem[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const processFile = async (filePath: string, fileName: string) => {
    const id = Math.random().toString(36).substr(2, 9)
    setQueue(prev => [...prev, { id, name: fileName, status: "processing", progress: 0 }])
    
    try {
      const result = await tauriInvoke("analyze_project_pdf", { filePath })
      setQueue(prev => prev.map(q => q.id === id ? { ...q, status: "done", resultId: result?.id, progress: 100 } : q))
    } catch (error) {
      setQueue(prev => prev.map(q => q.id === id ? { ...q, status: "error" } : q))
    }
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
          for (const path of files) {
            // Basit dosya adı çıkarımı
            const name = path.split('\\').pop()?.split('/').pop() || "Bilinmeyen PDF";
            processFile(path, name);
          }
        }
      } else {
        // Tarayıcı için Fallback: Gizli input tetikle
        fileInputRef.current?.click();
      }
    } catch (e) {
      console.warn("Tauri dialog failed, using fallback.", e);
      fileInputRef.current?.click();
    }
  }

  const handleFallbackChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      Array.from(e.target.files).forEach(file => {
        // Tarayıcıda tam dosya yolu alınamadığından mock için dosya adını kullanıyoruz
        processFile(file.name, file.name);
      });
      // inputu temizle
      e.target.value = '';
    }
  }

  return (
    <div className="flex flex-col gap-8">
      {/* Fallback Input */}
      <input 
        type="file" 
        multiple 
        accept=".pdf" 
        className="hidden" 
        ref={fileInputRef} 
        onChange={handleFallbackChange} 
      />

      <div 
        onClick={handleSelectFiles}
        onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
        onDragLeave={() => setIsDragging(false)}
        onDrop={(e) => {
          e.preventDefault();
          setIsDragging(false);
          if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
            Array.from(e.dataTransfer.files).forEach(file => processFile(file.name, file.name));
          }
        }}
        className={`bg-card rounded-2xl border-2 border-dashed p-10 lg:p-14 flex flex-col items-center justify-center text-center transition-all cursor-pointer
          ${isDragging ? 'border-primary bg-primary/10 scale-[1.02]' : 'border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/10'}`}
      >
        <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mb-6">
          <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-primary">
            <path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
            <polyline points="14 2 14 8 20 8" />
            <path d="M12 12v6" />
            <path d="m15 15-3-3-3 3" />
          </svg>
        </div>
        
        <h3 className="text-xl font-semibold mb-2">PDF Dosyalarını Sürükleyin</h3>
        <p className="text-muted-foreground mb-6 max-w-sm">
          veya bilgisayarınızdan toplu seçmek için tıklayın. Sadece .pdf formatı.
        </p>
        <button className="bg-primary text-primary-foreground hover:bg-primary/90 px-6 py-2.5 rounded-lg font-medium transition-colors pointer-events-none">
          Dosya Seç
        </button>
      </div>

      {/* Kuyruk (Queue) Listesi */}
      {queue.length > 0 && (
        <div className="bg-card border border-border rounded-xl shadow-sm overflow-hidden">
          <div className="bg-muted/30 px-5 py-3 border-b border-border flex items-center justify-between">
            <h4 className="font-semibold text-sm">İşlem Kuyruğu</h4>
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
                        {item.status === 'processing' ? 'KYS Motoru Analiz Ediyor...' : 
                         item.status === 'done' ? 'Analiz Tamamlandı' : 'Hata Oluştu'}
                      </span>
                    </div>
                  </div>
                </div>
                
                {item.status === 'done' && item.resultId && (
                  <a 
                    href={`/project?id=${item.resultId}`} 
                    className="text-xs bg-primary/10 text-primary hover:bg-primary/20 px-3 py-1.5 rounded-md font-medium transition-colors"
                  >
                    Detayı Gör
                  </a>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  )
}
