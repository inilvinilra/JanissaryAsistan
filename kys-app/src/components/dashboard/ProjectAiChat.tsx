import React, { useState, useRef, useEffect } from "react"
import { Send, Bot, User, Loader2 } from "lucide-react"

type Message = {
  id: string;
  role: "system" | "user" | "assistant";
  content: string;
}

export function ProjectAiChat({ projectTitle }: { projectTitle: string }) {
  const [messages, setMessages] = useState<Message[]>([
    {
      id: "1",
      role: "assistant",
      content: `Merhaba! Ben KYS Asistan. **"${projectTitle}"** projesinin analiz raporunu ve içeriğini inceledim. Projeyle ilgili ne öğrenmek istersiniz? (Örn: "Projenin zayıf yönleri neler?", "Benzerlik oranını nasıl düşürebilirim?")`
    }
  ])
  const [input, setInput] = useState("")
  const [isTyping, setIsTyping] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const handleSend = (e?: React.FormEvent) => {
    if (e) e.preventDefault()
    if (!input.trim()) return

    const userMessage: Message = { id: Date.now().toString(), role: "user", content: input.trim() }
    setMessages(prev => [...prev, userMessage])
    setInput("")
    setIsTyping(true)

    // Yapay Zeka Cevap Simülasyonu
    setTimeout(() => {
      setIsTyping(false)
      const aiMessage: Message = { 
        id: (Date.now() + 1).toString(), 
        role: "assistant", 
        content: "Şu an geliştirme aşamasındayım, ancak ilerleyen versiyonlarda bu projeye özel derinlemesine yapay zeka analizlerini, önerileri ve iyileştirme adımlarını burada seninle tartışıyor olacağım. Sorunu not aldım! 🚀" 
      }
      setMessages(prev => [...prev, aiMessage])
    }, 1500)
  }

  return (
    <div className="flex flex-col h-full bg-background text-foreground">
      {/* Messages Area */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6 custom-scrollbar">
        {messages.map((msg) => (
          <div key={msg.id} className={`flex gap-4 max-w-[85%] ${msg.role === 'user' ? 'ml-auto flex-row-reverse' : ''}`}>
            
            {/* Avatar */}
            <div className={`w-8 h-8 shrink-0 rounded-full flex items-center justify-center ${
              msg.role === 'user' ? 'bg-primary text-primary-foreground' : 'bg-muted text-foreground border border-border'
            }`}>
              {msg.role === 'user' ? <User className="w-5 h-5" /> : <Bot className="w-5 h-5" />}
            </div>

            {/* Message Bubble */}
            <div className={`p-3 rounded-2xl ${
              msg.role === 'user' 
                ? 'bg-primary text-primary-foreground rounded-tr-none' 
                : 'bg-muted border border-border text-foreground rounded-tl-none'
            }`}>
              <p className="text-sm whitespace-pre-wrap leading-relaxed">{msg.content}</p>
            </div>
            
          </div>
        ))}

        {isTyping && (
          <div className="flex gap-4 max-w-[85%]">
            <div className="w-8 h-8 shrink-0 rounded-full flex items-center justify-center bg-muted text-foreground border border-border">
              <Bot className="w-5 h-5" />
            </div>
            <div className="p-3 rounded-2xl bg-muted border border-border rounded-tl-none flex items-center gap-2">
              <Loader2 className="w-4 h-4 animate-spin text-muted-foreground" />
              <span className="text-xs text-muted-foreground">KYS Asistan düşünüyor...</span>
            </div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* Input Area */}
      <div className="p-4 bg-muted/30 border-t border-border">
        <form onSubmit={handleSend} className="relative flex items-end gap-2">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                handleSend()
              }
            }}
            placeholder="Proje hakkında soru sor..."
            className="w-full min-h-[50px] max-h-[150px] bg-background border border-input rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-1 focus:ring-primary/50 resize-none custom-scrollbar"
          />
          <button 
            type="submit" 
            disabled={!input.trim() || isTyping}
            className="h-[50px] w-[50px] shrink-0 bg-primary hover:bg-primary/90 text-primary-foreground rounded-xl flex items-center justify-center transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send className="w-5 h-5" />
          </button>
        </form>
        <p className="text-[10px] text-center text-muted-foreground mt-2">
          KYS AI Asistanı hata yapabilir. Lütfen önemli metrikleri doğrulayın.
        </p>
      </div>
    </div>
  )
}
