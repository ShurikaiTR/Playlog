// Turkish messages for the Rust API's error codes on the sign-in and sign-up pages.
const messages: Record<string, string> = {
	invalid_credentials: 'E-posta veya şifre hatalı.',
	invalid_email: 'Geçerli bir e-posta adresi gir.',
	invalid_password: 'Şifre en az 8 karakter olmalı.',
	invalid_username:
		'Kullanıcı adı 3-30 karakter olmalı; harf, rakam, alt çizgi ve nokta kullanabilirsin.',
	reserved_username: 'Bu kullanıcı adı kullanılamaz, başka bir tane dene.',
	invalid_display_name: 'Görünen ad en fazla 40 karakter olabilir.',
	username_taken: 'Bu kullanıcı adı alınmış, başka bir tane dene.',
	email_taken: 'Bu e-posta ile zaten bir hesap var. Giriş yapmayı dene.',
	invalid_code: 'Kod hatalı veya süresi dolmuş. Yeni bir kod isteyebilirsin.',
	code_cooldown: 'Yeni kod istemeden önce bir dakika bekle.',
	captcha_failed: 'Doğrulama başarısız oldu. Doğrulamayı tamamlayıp tekrar dene.',
	rate_limited: 'Çok fazla deneme yaptın. Biraz bekleyip tekrar dene.',
	oauth_failed: 'Giriş tamamlanamadı. Lütfen tekrar dene.',
	invalid_state: 'Giriş oturumunun süresi doldu. Lütfen tekrar dene.',
	invalid_pending: 'Kaydın süresi doldu. Lütfen baştan başla.',
	access_denied: 'Giriş iptal edildi.',
	email_unverified: 'Bu e-posta ile zaten bir hesap var. Önce e-postan ve şifrenle giriş yap.',
	unknown_provider: 'Bu giriş yöntemi şu an kullanılamıyor. E-posta ile devam edebilirsin.',
	unavailable: 'Sunucuya şu an ulaşılamıyor. Biraz sonra tekrar dene.'
};

export function authErrorMessage(code: string | undefined) {
	return (code && messages[code]) || 'Bir şeyler ters gitti. Lütfen tekrar dene.';
}
