import React, { useState } from "react"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  InputOTP,
  InputOTPGroup,
  InputOTPSlot,
} from "@/components/ui/input-otp"

// Tauri invoke - sadece masaüstü uygulamasında çalışır
async function tauriInvoke(cmd: string, args?: any): Promise<any> {
  if (typeof window !== "undefined" && (window as any).__TAURI__) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke(cmd, args);
  }
  // Tarayıcıda test modu: direkt başarılı döndür
  return { success: true, message: "Browser test mode" };
}

export function LoginCard() {
  const [isVerifying, setIsVerifying] = useState(false);
  const [otpValue, setOtpValue] = useState("");
  const [loading, setLoading] = useState(false);
  
  // Register state
  const [regName, setRegName] = useState("");
  const [regEmail, setRegEmail] = useState("");
  const [regPassword, setRegPassword] = useState("");

  const handleLogin = (e: React.FormEvent | React.MouseEvent) => {
    e.preventDefault();
    window.location.href = "/dashboard";
  };

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    try {
      const response = await tauriInvoke("register_user", {
        name: regName,
        email: regEmail,
        password: regPassword
      });
      
      if (response.success) {
        setIsVerifying(true);
      }
    } catch (error) {
      alert("Kayıt hatası: " + error);
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    if (otpValue.length === 6) {
      setLoading(true);
      try {
        const response = await invoke("verify_otp", {
          email: regEmail,
          otp: otpValue
        }) as any;
        
        if (response.success) {
          window.location.href = "/dashboard";
        }
      } catch (error) {
        alert("Doğrulama hatası: " + error);
      } finally {
        setLoading(false);
      }
    }
  };

  if (isVerifying) {
    return (
      <Card className="w-full max-w-sm shadow-xl">
        <CardHeader className="text-center">
          <CardTitle className="text-2xl">E-posta Doğrulama</CardTitle>
          <CardDescription>
            Lütfen e-posta adresinize gönderilen 6 haneli doğrulama kodunu girin.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex flex-col items-center gap-6">
          <InputOTP maxLength={6} value={otpValue} onChange={setOtpValue}>
            <InputOTPGroup>
              <InputOTPSlot index={0} />
              <InputOTPSlot index={1} />
              <InputOTPSlot index={2} />
              <InputOTPSlot index={3} />
              <InputOTPSlot index={4} />
              <InputOTPSlot index={5} />
            </InputOTPGroup>
          </InputOTP>
          <Button onClick={handleVerify} className="w-full" disabled={otpValue.length !== 6 || loading}>
            {loading ? "Doğrulanıyor..." : "Kodu Onayla"}
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Tabs defaultValue="login" className="w-full max-w-sm">
      <TabsList className="grid w-full grid-cols-2 mb-4">
        <TabsTrigger value="login">Giriş Yap</TabsTrigger>
        <TabsTrigger value="register">Kayıt Ol</TabsTrigger>
      </TabsList>

      <TabsContent value="login">
        <Card className="shadow-xl">
          <CardHeader>
            <CardTitle className="text-2xl">Jüri Girişi</CardTitle>
            <CardDescription>
              KYS hesabınıza erişmek için e-posta adresinizi girin.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleLogin}>
              <div className="flex flex-col gap-6">
                <div className="grid gap-2">
                  <Label htmlFor="email">E-posta</Label>
                  <Input id="email" type="email" placeholder="juri@kys.com" required />
                </div>
                <div className="grid gap-2">
                  <div className="flex items-center">
                    <Label htmlFor="password">Şifre</Label>
                    <a href="#" className="ml-auto inline-block text-sm underline-offset-4 hover:underline">
                      Şifremi unuttum?
                    </a>
                  </div>
                  <Input id="password" type="password" required />
                </div>
                <Button type="submit" className="w-full">
                  Giriş Yap
                </Button>
              </div>
            </form>
          </CardContent>
          <CardFooter>
            <Button variant="outline" className="w-full flex items-center justify-center gap-2" onClick={handleLogin}>
              <svg className="w-4 h-4" viewBox="0 0 24 24">
                <path fill="currentColor" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" />
                <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" />
                <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" />
                <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" />
              </svg>
              Google ile Devam Et
            </Button>
          </CardFooter>
        </Card>
      </TabsContent>

      <TabsContent value="register">
        <Card className="shadow-xl">
          <CardHeader>
            <CardTitle className="text-2xl">Hesap Oluştur</CardTitle>
            <CardDescription>
              KYS sistemine kayıt olmak için bilgilerinizi girin.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleRegister}>
              <div className="flex flex-col gap-6">
                <div className="grid gap-2">
                  <Label htmlFor="name">Ad Soyad</Label>
                  <Input id="name" type="text" placeholder="Emirhan" required value={regName} onChange={(e) => setRegName(e.target.value)} />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="reg-email">E-posta</Label>
                  <Input id="reg-email" type="email" placeholder="juri@kys.com" required value={regEmail} onChange={(e) => setRegEmail(e.target.value)} />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="reg-password">Şifre</Label>
                  <Input id="reg-password" type="password" required value={regPassword} onChange={(e) => setRegPassword(e.target.value)} />
                </div>
                <Button type="submit" className="w-full" disabled={loading}>
                  {loading ? "Hesap Oluşturuluyor..." : "Hesap Oluştur"}
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>
      </TabsContent>
    </Tabs>
  )
}
