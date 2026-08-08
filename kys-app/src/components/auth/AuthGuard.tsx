import React, { useEffect, useState } from "react"
import { supabase } from "@/lib/supabase"
import { signOut } from "@/lib/auth/authService"

export function AuthGuard({ children }: { children: React.ReactNode }) {
  const [checking, setChecking] = useState(true)
  const [authed, setAuthed] = useState(false)

  useEffect(() => {
    supabase.auth.getSession().then(({ data: { session } }) => {
      if (!session) {
        window.location.href = "/"
      } else {
        setAuthed(true)
        setChecking(false)
      }
    })

    const { data: { subscription } } = supabase.auth.onAuthStateChange((event, session) => {
      if (event === "SIGNED_OUT" || !session) {
        window.location.href = "/"
      }
    })

    return () => subscription.unsubscribe()
  }, [])

  if (checking) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-gray-400 animate-pulse">Kimlik doğrulanıyor...</div>
      </div>
    )
  }

  return <>{children}</>
}

export function LogoutButton() {
  const [loading, setLoading] = useState(false)

  const handleLogout = async () => {
    setLoading(true)
    await signOut()
    window.location.replace("/")
  }

  return (
    <button
      onClick={handleLogout}
      disabled={loading}
      className="text-sm text-red-600 hover:text-red-800 font-medium disabled:opacity-50"
    >
      {loading ? "Çıkış yapılıyor..." : "Çıkış Yap"}
    </button>
  )
}

export function UserInfo() {
  const [userEmail, setUserEmail] = useState<string | null>(null)
  const [fullName, setFullName] = useState<string | null>(null)

  useEffect(() => {
    supabase.auth.getUser().then(({ data: { user } }) => {
      if (user) {
        setUserEmail(user.email || null)
        setFullName(user.user_metadata?.full_name || null)
      }
    })
  }, [])

  return (
    <span className="text-sm text-gray-600">
      Hoş Geldiniz, <strong>{fullName || userEmail || "Kullanıcı"}</strong>
    </span>
  )
}

export default AuthGuard
