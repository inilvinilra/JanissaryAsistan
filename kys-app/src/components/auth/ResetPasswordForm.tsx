import React, { useState } from "react"
import { resetPassword } from "@/lib/auth/authService"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function ResetPasswordForm() {
  const [password, setPassword] = useState("")
  const [passwordConfirm, setPasswordConfirm] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [success, setSuccess] = useState("")

  const handleReset = async (e: React.FormEvent) => {
    e.preventDefault()
    if (password.length < 6) { setError("Şifre en az 6 karakter olmalıdır."); return }
    if (password !== passwordConfirm) { setError("Şifreler eşleşmiyor."); return }
    setLoading(true)
    setError("")
    const result = await resetPassword(password)
    setLoading(false)
    if (result.success) {
      setSuccess(result.message)
      setTimeout(() => { window.location.href = "/" }, 2000)
    } else {
      setError(result.message)
    }
  }

  if (success) {
    return (
      <Card className="w-full shadow-xl">
        <CardHeader>
          <CardTitle className="text-green-700">✅ {success}</CardTitle>
          <CardDescription>Giriş sayfasına yönlendiriliyorsunuz...</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <Card className="w-full shadow-xl">
      <CardHeader>
        <CardTitle className="text-2xl">Yeni Şifre Belirle</CardTitle>
        <CardDescription>Hesabınız için yeni bir şifre oluşturun.</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleReset} className="flex flex-col gap-4">
          <div className="grid gap-2">
            <Label htmlFor="new-password">Yeni Şifre</Label>
            <Input id="new-password" type="password" value={password} onChange={e => setPassword(e.target.value)} required />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="new-password-confirm">Yeni Şifre Tekrarı</Label>
            <Input id="new-password-confirm" type="password" value={passwordConfirm} onChange={e => setPasswordConfirm(e.target.value)} required />
          </div>
          {error && <p className="text-sm text-red-500 bg-red-50 p-3 rounded-md">{error}</p>}
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Şifre güncelleniyor..." : "Şifremi Güncelle"}
          </Button>
        </form>
      </CardContent>
    </Card>
  )
}

export default ResetPasswordForm
