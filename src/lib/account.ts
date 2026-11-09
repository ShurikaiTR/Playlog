// Shapes and Turkish copy for the account page (/hesap).

export type Provider = 'google' | 'discord' | 'apple';

export type Account = {
	id: string;
	username: string;
	display_name: string | null;
	email: string;
	avatar_url: string | null;
	created_at: string;
	has_password: boolean;
	sign_in_methods: { provider: Provider; created_at: string }[];
	/** Providers whose keys are configured, so they can be connected. */
	available: Record<Provider, boolean>;
};

export type Device = {
	id: string;
	user_agent: string | null;
	ip: string | null;
	created_at: string;
	last_seen_at: string;
	expires_at: string;
	current: boolean;
};

export const providers: { id: Provider; label: string }[] = [
	{ id: 'apple', label: 'Apple' },
	{ id: 'google', label: 'Google' },
	{ id: 'discord', label: 'Discord' }
];

export const providerLabel = (id: Provider) =>
	providers.find((provider) => provider.id === id)?.label ?? id;

export const providerSignInLabel = (id: Provider) => `${providerLabel(id)} ile giriş`;

const messages: Record<string, string> = {
	invalid_username:
		'Kullanıcı adı 3-20 karakter olmalı; küçük harf, rakam ve alt çizgi kullanabilirsin.',
	reserved_username: 'Bu kullanıcı adı kullanılamaz, başka bir tane dene.',
	username_taken: 'Bu kullanıcı adı alınmış, başka bir tane dene.',
	invalid_display_name: 'Görünen ad en fazla 40 karakter olabilir.',
	invalid_password: 'Mevcut şifre hatalı.',
	session_not_fresh:
		'Güvenliğin için çıkış yapıp yeniden giriş yap, sonra hesabını silmeyi tekrar dene.',
	last_sign_in_method:
		'Bu hesapta başka bir giriş yöntemin yok. Devam etmek için önce başka bir giriş yöntemi ekle.',
	last_admin: 'Son yönetici hesabı silinemez. Önce yönetici yetkisini başka birine ver.',
	email_taken: 'Bu e-posta başka bir hesapta kullanılıyor. Farklı bir adres dene.',
	invalid_email: 'Geçerli bir e-posta adresi gir.',
	invalid_code: 'Kod hatalı ya da süresi dolmuş. Yeni bir kod isteyebilirsin.',
	code_cooldown: 'Yeni kod istemeden önce bir dakika bekle.',
	provider_already_linked: 'Bu hesap zaten bağlı.',
	provider_linked_elsewhere: 'Bu hesap başka bir Playthru hesabına bağlı.',
	not_linked: 'Bu hesap bağlı değil.',
	session_not_found: 'Bu cihazın oturumu zaten kapanmış.',
	access_denied: 'Bağlama iptal edildi.',
	invalid_state: 'Bağlama süresi doldu. Lütfen tekrar dene.',
	oauth_failed: 'Hesap bağlanamadı. Lütfen tekrar dene.',
	unknown_provider: 'Bu giriş yöntemi şu an kullanılamıyor.',
	rate_limited: 'Çok fazla deneme yaptın. Biraz bekleyip tekrar dene.',
	unauthenticated: 'Oturumun kapanmış. Lütfen yeniden giriş yap.',
	unavailable: 'Sunucuya şu an ulaşılamıyor. Biraz sonra tekrar dene.'
};

export function accountErrorMessage(code: string | undefined) {
	return (code && messages[code]) || 'Bir şeyler ters gitti. Lütfen tekrar dene.';
}

/** A field-specific message for the new-password check, which shares a code with a wrong current password. */
export const NEW_PASSWORD_HINT = 'Şifre 8-128 karakter olmalı.';

/** Calls the account endpoints under /api/hesap; failures come back as an error code. */
export async function accountRequest(
	method: 'POST' | 'PUT' | 'DELETE',
	path: string,
	body?: unknown
): Promise<{ ok: true } | { ok: false; error: string }> {
	try {
		const response = await fetch(`/api/hesap${path}`, {
			method,
			headers: body === undefined ? undefined : { 'content-type': 'application/json' },
			body: body === undefined ? undefined : JSON.stringify(body)
		});
		if (response.ok) return { ok: true };
		const text = await response.text();
		const data = text ? JSON.parse(text) : null;
		return { ok: false, error: typeof data?.error === 'string' ? data.error : 'unknown' };
	} catch {
		return { ok: false, error: 'unavailable' };
	}
}

const systems: [RegExp, string][] = [
	[/iPhone/, 'iOS'],
	[/iPad/, 'iPadOS'],
	[/Android/, 'Android'],
	[/Mac OS X|Macintosh/, 'macOS'],
	[/Windows/, 'Windows'],
	[/CrOS/, 'ChromeOS'],
	[/Linux/, 'Linux']
];
// Order matters: Edge and Opera also say Chrome, and Chrome also says Safari.
const browsers: [RegExp, string][] = [
	[/Edg\//, 'Edge'],
	[/OPR\//, 'Opera'],
	[/Firefox\/|FxiOS/, 'Firefox'],
	[/Chrome\/|CriOS/, 'Chrome'],
	[/Safari\//, 'Safari']
];

/** Browser, operating system and form factor, read from the browser's user agent. */
export function describeDevice(userAgent: string | null) {
	const ua = userAgent ?? '';
	const os = systems.find(([pattern]) => pattern.test(ua))?.[1] ?? null;
	const browser = browsers.find(([pattern]) => pattern.test(ua))?.[1] ?? null;
	const kind: 'phone' | 'tablet' | 'computer' =
		/iPad/.test(ua) || (/Android/.test(ua) && !/Mobile/.test(ua))
			? 'tablet'
			: /iPhone|Mobile/.test(ua)
				? 'phone'
				: 'computer';
	return {
		/** "Safari · macOS" style name for lists. */
		name: [browser, os].filter(Boolean).join(' · ') || 'Bilinmeyen cihaz',
		browser: browser ?? 'Bilinmeyen tarayıcı',
		os: os ?? 'Bilinmeyen sistem',
		kind
	};
}

const dateFormat = new Intl.DateTimeFormat('tr', { dateStyle: 'long' });
const dateTimeFormat = new Intl.DateTimeFormat('tr', { dateStyle: 'medium', timeStyle: 'short' });

export const formatDate = (iso: string) => dateFormat.format(new Date(iso));
export const formatDateTime = (iso: string) => dateTimeFormat.format(new Date(iso));
const mediumDateFormat = new Intl.DateTimeFormat('tr', { dateStyle: 'medium' });
export const formatMediumDate = (iso: string) => mediumDateFormat.format(new Date(iso));
