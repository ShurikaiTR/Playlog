# Playthru: ürün planı ve tasarım

Hedef: oyunlar için IMDb/TMDB. Yani önce **güvenilir, eksiksiz bir oyun veritabanı**, onun üstünde
**kişisel takip** (kütüphane, puan, inceleme, liste) ve **topluluk**.

Bu belge sitenin bugünkü durumunu, hedeflenen yapıyı, veritabanını, API'yi, sayfa tasarımlarını ve
yapım sırasını anlatır. Aşama 1'in (katalog) kodu bu belgeyle birlikte gelir; sonrakiler plandır.

## 1. Bugünkü durum

**Gerçek olan:** kimlik doğrulama (e-posta, Google, Discord, Apple, Steam), hesap ayarları,
oturumlar, roller. Veritabanında yalnızca bu tablolar var.

**Maket olan:** oyunlar, listeler, puanlar, incelemeler, profiller. Hepsi `web/src/lib/features`
altında elle yazılmış örnek veri; yenileyince sıfırlanıyor.

İncelemede bulunanlar:

| Konu | Bulgu |
|---|---|
| Ölü bağlantılar | `/kutuphane`, `/incelemeler`, `/listeler`, `/kullanicilar`, `/blog`, `/topluluk`, `/fiyatlar` 404 veriyor |
| Hukuki | Kayıt ekranı "Terms of Service" ve "Privacy Policy"yi kabul ettiriyor; iki sayfa da yok (404) |
| Marka | Altı sayfanın başlığında ve kenar çubuğunda "Playlog"; ana sayfanın başlığı boş |
| Dil | Giriş/kayıt ve menü İngilizce, geri kalan Türkçe |
| Profil | Yalnızca başlık ve listeler var. Giriş yapılmamışken bile "Profili düzenle" görünüyor (izleyen kullanıcı sabit) |
| Oyun sayfası | Puan yalnızca inceleme penceresinin içinden verilebiliyor. Oynama süresi veride var, sayfada yok. Platformlar yalnızca en altta |
| Kaydetme menüsü | Düğme "Listeme ekle", menü "Kütüphanene ekle" diyor; durumlar ve listeler aynı menüde |
| Ana sayfa | Satırlar yedi oyunda ikinci satıra tek kart düşürüyor |
| Varlık sayfaları | Geliştirici, yayıncı, seri, tür, platform için sayfa yok; hepsi düz metin |

## 2. Kararlar

1. **Arayüz dili İngilizce.** URL'ler de İngilizce olur; eski Türkçe adresler kalıcı yönlendirilir.
   Çeviri altyapısı şimdi kurulmaz, ama metinler bileşenlere gömülü sabitler olarak değil, tek
   yerden değiştirilebilir halde tutulur. Dil seçici çeviri gelene kadar kaldırılır.
2. **Veri önce, sosyal sonra.** IMDb'yi IMDb yapan şey sayfalarının doğru ve eksiksiz olması.
   Her oyun, şirket, kişi, seri ve platformun kendi sayfası olur ve birbirine bağlanır.
3. **Olgular yapılandırılmış kaynaklardan gelir** (Wikidata, Steam, konsol özelleri için üçüncü
   bir kaynak). Dil modeli olgu üretmez. Her kayıt kaynağını ve dış kimliklerini taşır.
4. **Her değişiklik kayda geçer.** Başlangıçta yalnızca admin ve moderatör düzenler; ama düzenleme
   geçmişi ilk günden tutulur ki ileride TMDB gibi kullanıcı katkısına açılabilsin.
5. **Taslak → yayında.** İçe aktarılan kayıt taslak girer; yayına alınmadan sitede görünmez.
6. **Kimlik:** tablolar `uuidv7`, adresler `slug`. Slug değişirse eskisi yönlendirilir.
7. **Arama:** önce PostgreSQL (`pg_trgm`). Ayrı arama motoru ancak yetmediğinde.
8. **Görseller:** başlangıçta kaynağın adresi saklanır. Büyüyünce kendi depomuza (R2) kopyalanır;
   şema bunu şimdiden kaldırır (`url` alanı değişir, yapı değişmez).

## 3. Kapsam: sitenin sağladıkları

| Alan | Kullanıcı ne yapar |
|---|---|
| Katalog | Oyun, şirket, kişi, seri, platform, tür sayfalarında gezer; arar, süzer, sıralar |
| Oyun sayfası | Temel bilgi, çıkışlar (platform ve bölge), sürümler ve ek paketler, ekip, medya, dil desteği, sistem gereksinimi, bitirme süresi, puan dağılımı, incelemeler, benzer oyunlar |
| Kütüphane | Durum (Wishlist, Backlog, Playing, Paused, Completed, Dropped), puan, tarihler, sahiplik |
| İçe aktarma | Steam kütüphanesi ve oynama süreleri |
| İnceleme | Yazar, spoiler işaretler, başkasınınkini faydalı bulur |
| Listeler | Favoriler ve kendi listeleri; herkese açık ya da gizli |
| Profil | İstatistikler, favori oyunlar, şu an oynadıkları, kütüphane, incelemeler, listeler |
| Topluluk | Takip, akış, son incelemeler, popüler listeler |
| Bildirim | İstek listesindeki oyunun çıkışı, takip edilen serinin yeni oyunu |
| Katkı | Eksik/yanlış bilgi bildirir; ileride düzenleme önerir |
| Açık API | Sonraki aşama: TMDB'nin büyümesinin kaynağı buydu |

Kapsam dışı (bilerek): haber/blog, fiyat karşılaştırma, forum.

## 4. Veritabanı

Mevcut: `users`, `credentials`, `oauth_accounts`, `sessions`, `email_codes`.

### 4.1 Katalog (Aşama 1, migration `0008_catalog.sql`)

| Tablo | Amaç |
|---|---|
| `games` | Oyunun kendisi. `kind` (game, dlc, expansion, bundle, remake, remaster, port) ve `parent_id` ile ek paketler ana oyuna bağlanır |
| `series` | Seri/marka; `games.series_id` |
| `platforms` | PC, PS5, Switch… `family` ile gruplanır |
| `game_releases` | Oyun × platform × bölge için çıkış tarihi ve tarih kesinliği (gün, ay, çeyrek, yıl, belirsiz) |
| `tags`, `game_tags` | Tür, tema, oyun modu, bakış açısı tek tabloda `kind` ile |
| `companies`, `game_companies` | Geliştirici, yayıncı, port eden; rol ilişkide durur |
| `game_external_ids` | Steam appid, Wikidata QID… Aynı oyunun iki kez girmesini engeller |
| `game_media` | Ekran görüntüsü, görsel, fragman; sıralı |
| `game_languages` | Dil başına arayüz, seslendirme, altyazı |
| `game_age_ratings` | PEGI, ESRB… kurul başına |
| `game_alt_titles` | Diğer adlar ve yerel adlar (aramada kullanılır) |
| `game_slug_history` | Eski adreslerin yönlendirilmesi |

`games` üzerindeki önemli alanlar: `slug`, `title`, `summary`, `description`, `release_date` +
`release_precision` (en erken çıkış), `cover_url`, `backdrop_url`, `logo_url`,
`time_to_beat_minutes`, `pc_requirements` (jsonb), `status` (draft, published, hidden).

### 4.2 Kişisel takip (Aşama 2 ve 3)

```sql
CREATE TYPE play_status AS ENUM
    ('wishlist', 'backlog', 'playing', 'paused', 'completed', 'dropped');

-- Kullanıcı × oyun için tek kayıt. Satırın varlığı "kütüphanede" demektir
-- (durumu 'wishlist' olanlar istek listesidir).
CREATE TABLE user_games (
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id     uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    status      play_status,                       -- NULL: içe aktarılmış, henüz ayıklanmamış
    rating      smallint CHECK (rating BETWEEN 1 AND 10),
    started_at  date,
    finished_at date,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, game_id)
);

CREATE TABLE reviews (
    id                uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id           uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id           uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    body              text NOT NULL CHECK (char_length(body) BETWEEN 10 AND 10000),
    contains_spoilers boolean NOT NULL DEFAULT false,
    helpful_count     integer NOT NULL DEFAULT 0,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now(),
    UNIQUE (user_id, game_id)
);
CREATE TABLE review_votes (
    review_id uuid NOT NULL REFERENCES reviews (id) ON DELETE CASCADE,
    user_id   uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (review_id, user_id)
);

CREATE TYPE list_kind AS ENUM ('favorites', 'custom');
CREATE TABLE lists (
    id          uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind        list_kind NOT NULL DEFAULT 'custom',
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    description text,
    is_public   boolean NOT NULL DEFAULT false,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz NOT NULL DEFAULT now()
);
-- Her kullanıcının tek bir Favoriler listesi olur.
CREATE UNIQUE INDEX lists_one_favorites_idx ON lists (user_id) WHERE kind = 'favorites';
CREATE TABLE list_items (
    list_id  uuid NOT NULL REFERENCES lists (id) ON DELETE CASCADE,
    game_id  uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    position integer NOT NULL,
    note     text,
    added_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (list_id, game_id)
);
```

Puan özeti `games` üzerinde tutulur (`rating_count`, `rating_sum`, beş kovalı `rating_histogram`)
ve puan değişikliğiyle aynı transaction'da güncellenir. Böylece oyun sayfası her açılışta tüm
puanları saymaz.

### 4.3 Sahiplik ve içe aktarma (Aşama 5)

```sql
CREATE TABLE user_game_ownerships (
    user_id          uuid NOT NULL,
    game_id          uuid NOT NULL,
    platform_id      smallint NOT NULL REFERENCES platforms (id),
    source           text NOT NULL CHECK (source IN ('manual', 'steam')),
    playtime_minutes integer,
    last_played_at   timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, game_id, platform_id),
    FOREIGN KEY (user_id, game_id) REFERENCES user_games ON DELETE CASCADE
);

-- Kataloğumuzda olmayan ama kullanıcıların sahip olduğu oyunlar: içe aktarılacaklar kuyruğu.
CREATE TABLE catalog_requests (
    source          text NOT NULL,
    external_id     text NOT NULL,
    request_count   integer NOT NULL DEFAULT 1,
    first_requested timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source, external_id)
);
```

İçe aktarma kuralları: sahip olunan oyun için `user_games` satırı ve Steam sahipliği oluşur;
durum otomatik atanmaz; tek istisna `wishlist` → `backlog`. Yeniden içe aktarma yalnızca
`source = 'steam'` satırlarını günceller.

### 4.4 Topluluk (Aşama 4 ve 6)

| Tablo | Amaç |
|---|---|
| `follows` | `follower_id`, `followee_id` |
| `activities` | Akış: kim, ne yaptı (durum, puan, inceleme, liste), hangi oyun, ne zaman |
| `notifications` | Çıkış ve takip bildirimleri |
| `users` ek alanları | `bio`, `cover_url`, `is_private` |

### 4.5 IMDb derinliği (Aşama 7)

| Tablo | Amaç |
|---|---|
| `people`, `game_credits` | Yönetmen, yazar, besteci, seslendirme; kişi sayfaları |
| `characters`, `game_characters` | Karakterler ve seslendirenler |
| `revisions` | Her katalog değişikliği: kim, ne zaman, önce/sonra (jsonb) |
| `edit_suggestions` | Kullanıcı önerisi → moderatör onayı → `revisions` |
| `reports` | "Sorun bildir" formunun gittiği yer (bugün hiçbir yere gitmiyor) |
| `game_translations` | Dil başına başlık ve özet (TMDB modeli) |
| `time_to_beat_entries` | Bitirme süresini kullanıcı bildirimlerinden hesaplamak için |

## 5. API

Tarayıcı API'ye gitmez; SvelteKit sunucusu çağırır. Tüm uçlar `/v1` altında.

| Uç | Aşama | Açıklama |
|---|---|---|
| `GET /games` | 1 | Liste: `q`, `genres` (hepsi), `platforms` ve `modes` (herhangi biri), `year_min`, `year_max`, `window` (released/upcoming), `sort` (newest, soonest, title, added), `limit`, `offset` |
| `GET /tags?kind=genre` | 1 | Etiketler ve yayındaki oyun sayıları |
| `GET /games/{slug}` | 1 | Tüm ilişkileriyle oyun; eski slug'da yeni adresi döner |
| `POST /admin/games/import` | 1 | Toplu ekleme/güncelleme; dış kimliğe göre eşler |
| `PATCH /admin/games/{id}/status` | 1 | Taslak ↔ yayında ↔ gizli |
| `GET /admin/games/{slug}` | 1 | Taslaklar dahil önizleme (moderatör) |
| `GET /companies/{slug}`, `/series/{slug}`, `/tags/{kind}/{slug}`, `/platforms/{slug}` | 2 | Varlık sayfaları |
| `PUT/DELETE /me/games/{game_id}` | 2 | Durum, puan, tarihler |
| `GET /me/library` | 2 | Kütüphane; duruma ve platforma göre |
| `PUT/DELETE /me/games/{game_id}/review`, `PUT/DELETE /reviews/{id}/vote` | 3 | İnceleme ve faydalı oyu |
| `/me/lists`, `/lists/{id}`, `/lists/{id}/items` | 3 | Liste işlemleri |
| `GET /users/{username}` ve alt uçları | 4 | Profil, kütüphane, incelemeler, listeler |
| `PUT/DELETE /users/{username}/follow`, `GET /me/feed` | 4 | Takip ve akış |
| `POST /me/imports/steam` | 5 | Steam içe aktarma |

## 6. Sayfalar ve adresler

| Yeni adres | Eski | İçerik |
|---|---|---|
| `/` | `/` | Vitrin, trend, yakında çıkacaklar, takip edilenlerden hareketler |
| `/games` | `/oyunlar` | Tüm oyunlar, süzme ve sıralama |
| `/games/[slug]` | `/oyun/[slug]` | Oyun sayfası |
| `/discover` | `/kesfet` | Ruh haline göre keşif |
| `/genres`, `/genres/[slug]` | `/kesfet/turler` | Türler |
| `/companies/[slug]`, `/series/[slug]`, `/platforms/[slug]` | yok | Varlık sayfaları |
| `/library` | `/kutuphane` (404) | Kütüphanem |
| `/wishlist` | yok | İstek listem |
| `/lists/[id]` | `/liste/[id]`, `/favoriler` | Liste |
| `/u/[username]` | `/profil/[kullanici]` | Profil |
| `/reviews`, `/lists`, `/members` | 404 | Topluluk sayfaları |
| `/settings` | `/hesap` | Hesap ayarları |
| `/terms`, `/privacy` | 404 | Hukuki metinler |

### 6.1 Oyun sayfası

Mevcut düzen iyi; değişecekler:

1. **Başlık:** kapağın yanında platform rozetleri ve çıkış yılı.
2. **İşlem satırı:** durum düğmesi (yalnızca durumlar) · puan (1–10, doğrudan sayfada) · favori ·
   listeye ekle. Puan vermek için inceleme penceresini açmak gerekmez.
3. **Bilgi şeridi:** Puan · Bitirme süresi · Çıkış · Yaş · Mod · Dil. "Geliştirici" kutusu çıkar,
   çünkü aşağıda bağlantılı olarak var.
4. **"Your activity"** (giriş yapılmışsa): durum, puan, sahip olunan platformlar, oynama süresi,
   başlama/bitirme tarihi.
5. **Çıkışlar:** platform ve bölgeye göre tablo.
6. **Sürümler ve ek paketler:** `parent_id` üzerinden.
7. **Ekip:** geliştirici, yayıncı (bağlantılı), sonra kişiler.
8. **Takip ettiklerin:** kim oynadı, kaç verdi.
9. **Bu oyunun bulunduğu listeler.**

### 6.2 Profil sayfası

Bugün yalnızca başlık ve iki liste var. Hedef:

1. **Başlık:** avatar, ad, kullanıcı adı, biyografi, takip düğmesi; sayaçlar: Games · Reviews ·
   Lists · Following · Followers.
2. **Favori oyunlar:** en fazla beş kapak (Favoriler listesinin ilk beşi).
3. **Şu an oynadıkları.**
4. **İstatistik şeridi:** bu yıl bitirilen, toplam oyun, ortalama puan, toplam saat (Steam varsa).
5. **Sekmeler:** Overview · Library · Reviews · Lists · Wishlist.
6. **Overview:** son hareketler ve puan dağılımı.
7. Gizli profil: yalnızca başlık görünür.

### 6.3 Kütüphane

Durum sekmeleri (All, Playing, Backlog, Completed, Paused, Dropped), platform süzgeci, "Unsorted"
(durumu olmayan içe aktarılmışlar), kapak ızgarası ve tablo görünümü; kartta puan ve oynama süresi.

## 7. Aşamalar

| # | Aşama | Çıktı |
|---|---|---|
| 1 | **Katalog** | Katalog tabloları, içe aktarma ve okuma uçları. *Yapıldı.* |
| 1b | **Kataloğu arayüze bağlama** | Ana sayfa, oyunlar, oyun sayfası, keşfet, türler ve arama gerçek veriden; İngilizce adresler; veri betiği. *Yapıldı.* |
| 2 | Kütüphane | `user_games`, durum ve puan, `/library`, `/wishlist`, oyun sayfasında kişisel bölüm, varlık sayfaları |
| 3 | İnceleme ve listeler | Kalıcı incelemeler, faydalı oyu, listeler |
| 4 | Profil ve takip | Gerçek profil, takip, akış |
| 5 | Sahiplik ve Steam | İçe aktarma, oynama süreleri, katalog istek kuyruğu |
| 6 | Bildirim ve istatistik | Çıkış bildirimleri, yıl özeti |
| 7 | IMDb derinliği | Kişiler, karakterler, düzenleme geçmişi, katkı önerileri, açık API |

Aşamalardan bağımsız, hemen yapılabilecek temizlikler: ölü bağlantıları kaldırmak ya da sayfalarını
açmak, başlıklardaki "Playlog", `/terms` ve `/privacy`, dil seçicinin kaldırılması.

## 8. Veri akışı

Veriyi `tools/catalog/catalog.mjs` derler (Node 22+, bağımlılığı yok):

1. **Wikidata:** Steam'de olan ve Vikipedi'de en çok dilde yer alan oyunlar (popülerlik ölçüsü);
   her biri için ilk çıkış tarihi, platformlar ve seri.
2. **Steam:** açıklama, türler, oyun modları, geliştirici/yayıncı, dil desteği, yaş sınırları, PC
   gereksinimleri, ekran görüntüleri; kapak ve afiş adresleri Steam'in mağaza API'sinden.
3. Çıktı, içe aktarma ucunun biçiminde bir JSON dosyasıdır; `push` bunu yüzer yüzer gönderir.

```bash
# Sunucuda (compose ağı içinde)
docker compose run --rm catalog fetch --limit 500 --out /data/catalog.json
docker compose run --rm catalog push /data/catalog.json

# Yerelde
node tools/catalog/catalog.mjs fetch --limit 200 --out catalog.json
API_URL=http://localhost:8080 CATALOG_IMPORT_TOKEN=… node tools/catalog/catalog.mjs push catalog.json
```

`push`, API'nin `CATALOG_IMPORT_TOKEN` değeriyle kimlik doğrular; bu anahtar yalnızca içe aktarma
ucunda geçer. Aynı dosyayı yeniden göndermek kopya oluşturmaz, kayıtları günceller.

Bilinen sınırlar:

- Yalnızca Steam'de olan oyunlar gelir. Konsol özelleri için üçüncü bir kaynak gerekiyor.
- Platform başına çıkış tarihi yok; yalnızca ilk çıkış tarihi biliniyor.
- Steam hangi dillerde altyazı olduğunu ayırmıyor; yalnızca arayüz ve seslendirme geliyor.
- Fragman ve bitirme süresi henüz yok; sayfalar bu alanları veri olunca gösterir.
- Wikidata yoğun adresleri dakikada bir isteğe düşürebiliyor; betik bekleyip yeniden dener.
- Kaynak yanıtları önbelleğe alınır: yerelde `tools/catalog/.cache`, sunucuda `catalogdata`
  volume'u (`/data/.cache`). Taze veri için önbelleği silin.

### İçe aktarma biçimi

`POST /v1/admin/games/import` (admin), tek istekte en fazla 500 oyun. Yalnızca `title` zorunludur.

```json
{
  "games": [
    {
      "title": "The Witcher 3: Wild Hunt",
      "status": "published",
      "series": "The Witcher",
      "summary": "A story-driven open world RPG.",
      "cover_url": "https://…/cover.jpg",
      "time_to_beat_minutes": 3090,
      "releases": [
        { "platform": "pc", "date": "2015-05-19" },
        { "platform": "switch", "date": "2019-10-15" },
        { "platform": "ps5", "date": "2027-01-01", "precision": "year" }
      ],
      "genres": ["RPG", "Action"],
      "themes": ["Fantasy"],
      "modes": ["Single player"],
      "developers": ["CD Projekt Red"],
      "publishers": ["CD Projekt"],
      "external_ids": { "steam": "292030", "wikidata": "Q11163283" },
      "screenshots": ["https://…/1.jpg"],
      "trailers": [{ "provider": "youtube", "video_id": "c0i88t0Kacs" }],
      "languages": [{ "language": "tr", "interface": true, "subtitles": true }],
      "age_ratings": { "pegi": "18" },
      "alt_titles": ["Wiedźmin 3: Dziki Gon"]
    }
  ]
}
```

Kurallar:

- Oyun önce `external_ids`, sonra `slug` ile eşlenir; eşleşirse güncellenir, yoksa eklenir.
- Güncelleme oyunun tam halini yazar: ilişkiler (çıkışlar, etiketler, şirketler, medya, diller)
  gönderilenle değiştirilir. Dış kimlikler silinmez, yalnızca eklenir.
- Aynı adlı farklı oyun (aynı kaynakta başka kimlik) ayrı kayıt olur; slug'ına çıkış yılı eklenir.
- `status` verilmezse yeni oyun `draft` girer, var olanın durumu korunur.
- Ek paketlerde `kind` ve `parent_slug` verilir; ana oyun önce eklenmiş olmalıdır.
- Görsel adresleri yalnızca `https` olabilir. Platformlar `platforms` tablosundaki slug'lardır.
- Her oyun ayrı işlenir; yanıt her biri için `created`, `updated` ya da `failed` ve hata kodunu verir.

