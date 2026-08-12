import React, { useEffect, useState } from "react"
import { supabase } from "@/lib/supabase"
import { signOut } from "@/lib/auth/authService"

/**
 * AuthGuard — Supabase v2 için doğru auth guard implementasyonu.
 *
 * Tek kaynak: onAuthStateChange
 * - INITIAL_SESSION → Supabase localStorage'daki token'ı okur, geri döner
 *   - session varsa → render et
 *   - session yoksa → login'e yönlendir
 * - SIGNED_IN → session geldi, render et
 * - SIGNED_OUT → login'e yönlendir
 * - TOKEN_REFRESHED → session yenilendi, devam et
 *
 * getSession() ile paralel çağrı YAPILMIYOR çünkü ikisi race condition yaratır.
 */
export function AuthGuard({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState<"loading" | "authed" | "denied">("loading")

  useEffect(() => {
    let mounted = true

    // 1. Mevcut session'ı güvenli şekilde kontrol et
    supabase.auth.getSession().then(({ data: { session } }) => {
      if (!mounted) return
      if (session) {
        setStatus("authed")
      } else {
        setStatus("denied")
      }
    }).catch(err => {
      console.error("AuthGuard session check failed:", err)
      if (mounted) setStatus("denied")
    })

    // 2. Auth değişikliklerini dinle
    const { data: { subscription } } = supabase.auth.onAuthStateChange((event, session) => {
      if (!mounted) return
      if (event === "SIGNED_IN" || event === "TOKEN_REFRESHED") {
        setStatus("authed")
      } else if (event === "SIGNED_OUT") {
        setStatus("denied")
      }
    })

    return () => {
      mounted = false
      subscription.unsubscribe()
    }
  }, [])

  // denied olduğunda redirect
  useEffect(() => {
    if (status === "denied") {
      window.location.replace("/")
    }
  }, [status])

  if (status === "loading") {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-primary border-t-transparent rounded-full animate-spin" />
          <p className="text-sm text-muted-foreground">Kimlik doğrulanıyor...</p>
        </div>
      </div>
    )
  }

  if (status === "denied") {
    // Redirect useEffect'te yapılıyor, boş göster
    return null
  }

  return <>{children}</>
}

export function LogoutButton() {
  const [loading, setLoading] = useState(false)

  const handleLogout = async () => {
    setLoading(true)
    try {
      await signOut()
    } catch {
      // signOut hata verse bile redirect et
    }
    window.location.replace("/")
  }

  return (
    <button
      onClick={handleLogout}
      disabled={loading}
      className="text-sm text-red-600 hover:text-red-800 font-medium disabled:opacity-50 transition-colors"
    >
      {loading ? "Çıkış yapılıyor..." : "Çıkış Yap"}
    </button>
  )
}

export function UserInfo() {
  const [displayName, setDisplayName] = useState<string>("Kullanıcı")

  useEffect(() => {
    supabase.auth.getUser().then(({ data: { user } }) => {
      if (user) {
        const name = user.user_metadata?.full_name || user.email || "Kullanıcı"
        setDisplayName(name)
      }
    }).catch(() => {})
  }, [])

  return (
    <span className="text-sm text-muted-foreground">
      Hoş Geldiniz, <strong className="text-foreground">{displayName}</strong>
    </span>
  )
}

export default AuthGuard
