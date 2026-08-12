import React, { useState } from "react"
import { signUp } from "@/lib/auth/authService"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function RegisterForm() {
  const [fullName, setFullName] = useState("")
  const [email, setEmail] = useState("")
  const [password, setPassword] = useState("")
  const [passwordConfirm, setPasswordConfirm] = useState("")
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState("")
  const [success, setSuccess] = useState("")

  const validate = () => {
    if (!fullName.trim()) return "Ad Soyad alanı zorunludur."
    if (!email.match(/^[^\s@]+@[^\s@]+\.[^\s@]+$/)) return "Geçerli bir e-posta adresi girin."
    if (password.length < 6) return "Şifre en az 6 karakter olmalıdır."
    if (password !== passwordConfirm) return "Şifreler eşleşmiyor."
    return null
  }

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault()
    const validationError = validate()
    if (validationError) { setError(validationError); return }
    setLoading(true)
    setError("")
    const result = await signUp(fullName, email, password)
    setLoading(false)
    if (result.success) {
      setSuccess(result.message)
    } else {
      setError(result.message)
    }
  }

  if (success) {
    return (
      <Card className="w-full shadow-xl">
        <CardHeader>
          <CardTitle className="text-2xl text-green-700">✉️ E-posta Doğrulama</CardTitle>
          <CardDescription>{success}</CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <p className="text-sm text-gray-600">
            <strong>{email}</strong> adresine bir doğrulama bağlantısı gönderdik. 
            Bağlantıya tıkladıktan sonra giriş yapabilirsiniz.
          </p>
          <p className="text-xs text-gray-400">Spam klasörünü de kontrol etmeyi unutmayın.</p>
        </CardContent>
        <CardFooter>
          <a href="/" className="text-sm text-blue-600 hover:underline">← Giriş sayfasına dön</a>
        </CardFooter>
      </Card>
    )
  }

  return (
    <Card className="w-full shadow-xl">
      <CardHeader>
        <CardTitle className="text-2xl">Hesap Oluştur</CardTitle>
        <CardDescription>JanissaryAsistan sistemine kayıt olmak için bilgilerinizi girin.</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleRegister} className="flex flex-col gap-4">
          <div className="grid gap-2">
            <Label htmlFor="fullname">Ad Soyad</Label>
            <Input id="fullname" type="text" placeholder="Emirhan Yazıcı" value={fullName} onChange={e => setFullName(e.target.value)} required />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="reg-email">E-posta</Label>
            <Input id="reg-email" type="email" placeholder="juri@kys.com" value={email} onChange={e => setEmail(e.target.value)} required />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="reg-password">Şifre <span className="text-xs text-gray-400">(min. 6 karakter)</span></Label>
            <Input id="reg-password" type="password" value={password} onChange={e => setPassword(e.target.value)} required />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="reg-password-confirm">Şifre Tekrarı</Label>
            <Input id="reg-password-confirm" type="password" value={passwordConfirm} onChange={e => setPasswordConfirm(e.target.value)} required />
          </div>
          {error && <p className="text-sm text-red-500 bg-red-50 p-3 rounded-md">{error}</p>}
          <Button type="submit" className="w-full" disabled={loading}>
            {loading ? "Hesap oluşturuluyor..." : "Hesap Oluştur"}
          </Button>
        </form>
      </CardContent>
      <CardFooter className="justify-center">
        <p className="text-sm text-gray-500">
          Zaten hesabın var mı?{" "}
          <a href="/" className="text-blue-600 hover:underline font-medium">Giriş Yap</a>
        </p>
      </CardFooter>
    </Card>
  )
}

export default RegisterForm
