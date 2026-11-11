# Playthru

Oyun takip ve keşif sitesi: [playthru.app](https://playthru.app)

İki servis ve bir veritabanından oluşur. Tarayıcı yalnızca web sunucusuyla konuşur; web sunucusu
Rust API'yi iç ağdan çağırır.

```
tarayıcı ──> Cloudflare Tunnel ──> web (SvelteKit, SSR) ──> api (Rust, Axum) ──> PostgreSQL 18
```

## Klasör yapısı

```
.
├── compose.yaml        Üretim: web + api + db (Openship bunu çalıştırır)
├── .env.example        Üretim sırları (compose.yaml okur)
├── web/                SvelteKit uygulaması (adapter-node, Tailwind, shadcn-svelte)
│   ├── src/
│   │   ├── routes/         Sayfalar ve sunucu uçları
│   │   ├── hooks.server.ts Oturum çerezinden kullanıcıyı çözer
│   │   └── lib/
│   │       ├── features/       Özellik başına klasör: bileşen, durum ve yardımcılar bir arada
│   │       │   ├── auth/       Giriş/kayıt akışları, sağlayıcılar, yönlendirme güvenliği
│   │       │   ├── account/    /hesap sayfası: tipler, istekler, şifre ve silme diyalogları
│   │       │   ├── settings/   Ayarlar penceresi, dil ve geri bildirim
│   │       │   ├── games/      Oyun verisi, kart, satır, galeri
│   │       │   ├── lists/      Kullanıcı listeleri, kaydetme menüsü
│   │       │   ├── profiles/   Profil verisi
│   │       │   ├── home/       Ana sayfa vitrini
│   │       │   └── search/     Arama penceresi
│   │       ├── components/
│   │       │   ├── ui/         shadcn-svelte bileşenleri (elle düzenlenmez)
│   │       │   ├── icons/      Marka ve platform ikonları
│   │       │   └── layout/     Uygulama kabuğu, kenar çubuğu, alt bilgi, duyarlı panel
│   │       ├── server/         Yalnızca sunucuda çalışan kod: API istemcisi, oturum, OAuth
│   │       └── hooks/          Paylaşılan reaktif yardımcılar
│   └── static/             Olduğu gibi sunulan dosyalar
└── api/                Rust API (Axum, SQLx)
    ├── migrations/         SQL migration'ları; API açılışta uygular
    └── src/
        ├── main.rs         Yapılandırma, havuz, migration, sunucu
        ├── config.rs       Ortam değişkenleri
        ├── error.rs        AppError ve HTTP karşılıkları
        ├── state.rs        Paylaşılan uygulama durumu
        ├── auth/           Kimlik doğrulamanın çekirdeği (HTTP'den bağımsız parçalar)
        │                   şifre, oturum belirteci, oturum kayıtları, e-posta kodları, roller, doğrulama
        ├── oauth/          Sağlayıcı girişi: ortak akış, imzalı belirteçler, sağlayıcı başına dosya
        ├── routes/         HTTP uçları
        │   ├── auth/       /v1/auth: kayıt, giriş, şifre sıfırlama
        │   ├── account/    /v1/account: profil, şifre, oturumlar, giriş yöntemleri, silme, dışa aktarma
        │   ├── oauth.rs    /v1/auth/oauth
        │   ├── admin.rs    /v1/admin
        │   └── guard.rs    İstemci IP'si, hız sınırı, Turnstile
        ├── mail.rs         Resend ile e-posta
        ├── ratelimit.rs    Bellek içi hız sınırlayıcı
        └── turnstile.rs    Cloudflare Turnstile doğrulaması
```

Yeni bir özellik eklerken: arayüz tarafı `web/src/lib/features/<özellik>/` altına, API tarafı
`api/src/routes/<özellik>` altına gider. Birden fazla ucun paylaştığı iş mantığı route dosyasında
değil, kendi modülünde durur (`auth/`, `oauth/` gibi).

## Yerel geliştirme

Gerekenler: Node 22+, Rust 1.94+, Docker (yalnızca veritabanı için).

```bash
# 1) Veritabanı (127.0.0.1:5433)
docker compose -f api/compose.yaml up -d

# 2) API (127.0.0.1:8080); migration'lar açılışta uygulanır
cd api
cp .env.example .env
cargo run

# 3) Web (localhost:5173)
cd web
cp .env.example .env
npm install
npm run dev
```

`RESEND_API_KEY` tanımlı değilse e-postalar gönderilmez, API günlüğüne yazılır; doğrulama
kodlarını oradan okuyabilirsin.

## Denetimler

Aşağıdaki komutların hepsini GitHub Actions her PR'da ve `main`'e her push'ta çalıştırır
(`.github/workflows/ci.yml`).

```bash
# web/
npm run check     # tip denetimi
npm run lint      # prettier + eslint
npm run build

# api/
cargo fmt --check
cargo clippy --all-targets
cargo test
```

## Yayın

VPS'te Openship, kökteki `compose.yaml` dosyasını çalıştırır. Sırlar `.env` dosyasından ya da
panelden gelir (`.env.example` listeyi verir). Yalnızca web dışarı açıktır (`127.0.0.1:3000`,
önünde Cloudflare Tunnel); API ve veritabanı yalnızca compose iç ağındadır.
