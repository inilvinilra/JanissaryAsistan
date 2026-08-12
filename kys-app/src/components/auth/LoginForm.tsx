import React, { useState } from "react"
import { signIn, forgotPassword } from "@/lib/auth/authService"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function LoginForm() {
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [showForgot, setShowForgot] = useState(false)
  const [forgotEmail, setForgotEmail] = useState("")
  const [forgotLoading, setForgotLoading] = useState(false)
  const [forgotMsg, setForgotMsg] = useState("")

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!email || !password) { setError("Lütfen tüm alanları doldurun."); return }
    setLoading(true)
    setError("")
    const result = await signIn(email, password)
    setLoading(false)
    if (result.success) {
      window.location.replace("/dashboard")
    } else {
      setError(result.message)
    }
  }

  const handleForgot = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!forgotEmail) { setForgotMsg("E-posta adresi girin."); return }
    setForgotLoading(true)
    const result = await forgotPassword(forgotEmail)
    setForgotLoading(false)
    setForgotMsg(result.message)
  }

  if (showForgot) {
    return (
      <Card className="w-full shadow-xl">
        <CardHeader>
          <CardTitle className="text-2xl">Şifremi Unuttum</CardTitle>
          <CardDescription>E-posta adresinize sıfırlama bağlantısı göndereceğiz.</CardDescription>
        </CardHeader>
        <CardContent>
          <form onSubmit={handleForgot} className="flex flex-col gap-4">
            <div className="grid gap-2">
              <Label htmlFor="forgot-email">E-posta</Label>
              <Input id="forgot-email" type="email" placeholder="juri@kys.com" value={forgotEmail} onChange={e => setForgotEmail(e.target.value)} required />
            </div>
            {forgotMsg && (
              <p className={`text-sm ${forgotMsg.includes('gönderildi') ? 'text-green-600' : 'text-red-500'}`}>{forgotMsg}</p>
            )}
            <Button type="submit" className="w-full" disabled={forgotLoading}>
              {forgotLoading ? "Gönderiliyor..." : "Sıfırlama Bağlantısı Gönder"}
            </Button>
            <Button type="button" variant="ghost" className="w-full" onClick={() => setShowForgot(false)}>
              ← Giriş Yap'a Dön
            </Button>
          </form>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="w-full shadow-xl">
      <CardHeader>
        <CardTitle className="text-2xl">Jüri Girişi</CardTitle>
        <CardDescription>JanissaryAsistan hesabınıza erişmek için bilgilerinizi girin.</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleLogin} className="flex flex-col gap-4">
          <div className="grid gap-2">
            <Label htmlFor="email">E-posta</Label>
            <Input id="email" type="email" placeholder="juri@kys.com" value={email} onChange={e => setEmail(e.target.value)} required />
          </div>
          <div className="grid gap-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="password">Şifre</Label>
              <button type="button" onClick={() => setShowForgot(true)} className="text-sm text-blue-600 hover:underline">
                Şifremi unuttum?
              </button>
            </div>
            <Input id="password" type="password" value={password} onChange={e => setPassword(e.target.value)} required />
          </div>
          {error && <p className="text-sm text-red-500 bg-red-50 p-3 rounded-md">{error}</p>}
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Giriş yapılıyor..." : "Giriş Yap"}
          </Button>
        </form>
      </CardContent>
      <CardFooter className="justify-center">
        <p className="text-sm text-gray-500">
          Hesabın yok mu?{" "}
          <a href="/register" className="text-blue-600 hover:underline font-medium">Kayıt Ol</a>
        </p>
      </CardFooter>
    </Card>
  )
}

export default LoginForm
