import React, { useState, useEffect } from "react"
import { Bot, Send, Loader2 } from "lucide-react"
import { tauriInvoke } from "@/lib/tauri"

interface Message {
  role: "user" | "assistant"
  content: string
}

interface ProjectCopilotProps {
  projectId: string
  projectTitle: string
  initialContext?: string
}

export function ProjectCopilot({ projectId, projectTitle, initialContext }: ProjectCopilotProps) {
  const [messages, setMessages] = useState<Message[]>([
    {
      role: "assistant",
      content: `Merhaba! Ben Janissary Copilot. "${projectTitle}" projesi hakkında sana nasıl yardımcı olabilirim? Örneğin "Bu projenin zayıf yönleri nelerdir?" diye sorabilirsin.`
    }
  ])
  const [input, setInput] = useState("")
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (initialContext) {
      setInput(`PDF'deki edilen kelime analizi: "${initialContext}"`)
    }
  }, [initialContext])

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!input.trim()) return

    const userMessage = input.trim()
    setMessages(prev => [...prev, { role: "user", content: userMessage }])
    setInput("")
    setLoading(true)

    try {
      const response = await tauriInvoke("ask_copilot", {
        projectId,
        projectTitle,
        message: userMessage,
        context: initialContext || null
      }) as string;
      
      setMessages(prev => [
        ...prev,
        { role: "assistant", content: response }
      ])
    } catch (error: any) {
      setMessages(prev => [
        ...prev,
        { role: "assistant", content: `Hata oluştu: ${error}` }
      ])
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex flex-col h-full bg-background text-foreground relative">
      {/* Mesaj Listesi */}
      <div className="flex-1 overflow-y-auto p-4 space-y-4 custom-scrollbar">
        {messages.map((msg, idx) => (
          <div key={idx} className={`flex gap-3 max-w-[85%] ${msg.role === "user" ? "ml-auto flex-row-reverse" : ""}`}>
            <div className={`w-8 h-8 rounded-full flex items-center justify-center shrink-0 ${msg.role === "assistant" ? "bg-primary/20 text-primary" : "bg-muted text-muted-foreground"}`}>
              {msg.role === "assistant" ? <Bot className="w-5 h-5" /> : <div className="text-xs font-bold">Sen</div>}
            </div>
            <div className={`p-3 rounded-2xl text-sm ${msg.role === "user" ? "bg-primary text-primary-foreground rounded-tr-sm" : "bg-muted/50 border border-border rounded-tl-sm text-foreground"}`}>
              {msg.content}
            </div>
          </div>
        ))}
        {loading && (
          <div className="flex gap-3 max-w-[85%]">
            <div className="w-8 h-8 rounded-full flex items-center justify-center shrink-0 bg-primary/20 text-primary">
              <Bot className="w-5 h-5" />
            </div>
            <div className="p-4 rounded-2xl bg-muted/50 border border-border rounded-tl-sm flex items-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin text-primary" />
              <span className="text-xs text-muted-foreground">Copilot düşünüyor...</span>
            </div>
          </div>
        )}
      </div>

      {/* Mesaj Gönderme Alanı */}
      <div className="p-3 bg-card border-t border-border shrink-0">
        <form onSubmit={handleSend} className="relative flex items-center">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            placeholder={`${projectTitle} hakkında bir soru sor...`}
            className="w-full h-11 pl-4 pr-12 bg-background border border-border rounded-full text-sm focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-all"
            disabled={loading}
          />
          <button
            type="submit"
            disabled={!input.trim() || loading}
            className="absolute right-1.5 w-8 h-8 flex items-center justify-center bg-primary hover:bg-primary/90 text-primary-foreground rounded-full transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send className="w-4 h-4 ml-0.5" />
          </button>
        </form>
        <div className="text-[10px] text-center text-muted-foreground mt-2">
          Janissary Copilot hata yapabilir. Lütfen önemli kararlarda verileri doğrulayın.
        </div>
      </div>
    </div>
  )
}
