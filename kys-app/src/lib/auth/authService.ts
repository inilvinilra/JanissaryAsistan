import { supabase } from '../supabase'

// Türkçe hata mesajları
function humanizeError(error: any): string {
  const msg = error?.message || ''
  if (msg.includes('User already registered')) return 'Bu e-posta adresiyle zaten bir hesap bulunuyor.'
  if (msg.includes('Invalid login credentials')) return 'E-posta veya şifre hatalı.'
  if (msg.includes('Email not confirmed')) return 'E-posta adresiniz henüz doğrulanmadı. Gelen kutunuzu kontrol edin.'
  if (msg.includes('Password should be at least')) return 'Şifre en az 6 karakter olmalıdır.'
  if (msg.includes('Unable to validate email address')) return 'Geçersiz e-posta adresi.'
  if (msg.includes('signup is disabled')) return 'Kayıt şu an kapalı. Yönetici ile iletişime geçin.'
  if (msg.includes('Email rate limit exceeded')) return 'Çok fazla istek gönderildi. Lütfen bekleyin.'
  return 'Bir hata oluştu. Lütfen tekrar deneyin.'
}

export type AuthResult = {
  success: boolean
  message: string
  data?: any
}

// Kayıt Ol
export async function signUp(fullName: string, email: string, password: string): Promise<AuthResult> {
  try {
    const { data, error } = await supabase.auth.signUp({
      email,
      password,
      options: {
        data: { full_name: fullName },
        emailRedirectTo: `${window.location.origin}/dashboard`,
      }
    })
    if (error) return { success: false, message: humanizeError(error) }
    return { success: true, message: 'Doğrulama e-postası gönderildi. Lütfen e-postanızı kontrol edin.', data }
  } catch (e) {
    return { success: false, message: 'Bağlantı hatası. İnternet bağlantınızı kontrol edin.' }
  }
}

// Giriş Yap
export async function signIn(email: string, password: string): Promise<AuthResult> {
  try {
    const { data, error } = await supabase.auth.signInWithPassword({ email, password })
    if (error) return { success: false, message: humanizeError(error) }
    if (!data.user?.email_confirmed_at) {
      await supabase.auth.signOut()
      return { success: false, message: 'E-posta adresiniz doğrulanmadı. Gelen kutunuzu kontrol edin.' }
    }
    return { success: true, message: 'Giriş başarılı!', data }
  } catch (e) {
    return { success: false, message: 'Bağlantı hatası. İnternet bağlantınızı kontrol edin.' }
  }
}

// Çıkış Yap
export async function signOut(): Promise<AuthResult> {
  try {
    const { error } = await supabase.auth.signOut()
    if (error) return { success: false, message: humanizeError(error) }
    return { success: true, message: 'Çıkış yapıldı.' }
  } catch (e) {
    return { success: false, message: 'Çıkış yapılırken hata oluştu.' }
  }
}

// Şifremi Unuttum
export async function forgotPassword(email: string): Promise<AuthResult> {
  try {
    const { error } = await supabase.auth.resetPasswordForEmail(email, {
      redirectTo: `${window.location.origin}/reset-password`,
    })
    if (error) return { success: false, message: humanizeError(error) }
    return { success: true, message: 'Şifre sıfırlama bağlantısı e-posta adresinize gönderildi.' }
  } catch (e) {
    return { success: false, message: 'Bağlantı hatası.' }
  }
}

// Şifre Sıfırla
export async function resetPassword(newPassword: string): Promise<AuthResult> {
  try {
    const { error } = await supabase.auth.updateUser({ password: newPassword })
    if (error) return { success: false, message: humanizeError(error) }
    return { success: true, message: 'Şifreniz başarıyla güncellendi.' }
  } catch (e) {
    return { success: false, message: 'Şifre güncellenirken hata oluştu.' }
  }
}

// Mevcut oturumu getir
export async function getSession() {
  const { data: { session } } = await supabase.auth.getSession()
  return session
}

// Doğrulama e-postasını tekrar gönder
export async function resendVerification(email: string): Promise<AuthResult> {
  try {
    const { error } = await supabase.auth.resend({ type: 'signup', email })
    if (error) return { success: false, message: humanizeError(error) }
    return { success: true, message: 'Doğrulama e-postası tekrar gönderildi.' }
  } catch (e) {
    return { success: false, message: 'Gönderme hatası.' }
  }
}
