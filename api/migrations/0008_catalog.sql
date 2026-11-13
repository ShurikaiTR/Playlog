-- Oyun kataloğu (docs/plan.md, Aşama 1): oyunlar, platformlar, çıkışlar, etiketler, şirketler, medya.
-- Kişisel takip (kütüphane, puan, inceleme, liste) sonraki aşamalarda bu tablolara bağlanır.

-- Başlık araması için trigram indeksi ("witcer" yazan "The Witcher"ı bulur).
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TYPE game_kind AS ENUM ('game', 'dlc', 'expansion', 'bundle', 'remake', 'remaster', 'port');
-- İçe aktarılan kayıt taslak girer; yalnızca 'published' olanlar sitede görünür.
CREATE TYPE publish_status AS ENUM ('draft', 'published', 'hidden');
-- Çıkış tarihleri çoğu zaman kesin değildir ("2027", "Mart 2027"). Tarih, dönemin ilk günü olarak
-- saklanır; kesinlik nasıl gösterileceğini söyler. 'tba': tarih yok.
CREATE TYPE date_precision AS ENUM ('day', 'month', 'quarter', 'year', 'tba');
CREATE TYPE tag_kind AS ENUM ('genre', 'theme', 'mode', 'perspective');
CREATE TYPE company_role AS ENUM ('developer', 'publisher', 'porting', 'supporting');
CREATE TYPE media_kind AS ENUM ('screenshot', 'artwork', 'trailer');

-- Adreslerde kullanılan kısa ad: küçük harf, rakam ve tek tire.
CREATE DOMAIN slug AS text CHECK (VALUE ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(VALUE) <= 120);

CREATE TABLE series (
    id         uuid PRIMARY KEY DEFAULT uuidv7(),
    slug       slug NOT NULL UNIQUE,
    name       text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE games (
    id                   uuid PRIMARY KEY DEFAULT uuidv7(),
    slug                 slug NOT NULL UNIQUE,
    title                text NOT NULL CHECK (char_length(title) BETWEEN 1 AND 300),
    kind                 game_kind NOT NULL DEFAULT 'game',
    -- Ek paket, yeniden yapım ya da port ise ana oyun.
    parent_id            uuid REFERENCES games (id) ON DELETE SET NULL,
    series_id            uuid REFERENCES series (id) ON DELETE SET NULL,
    -- Kartlarda ve arama sonuçlarında görünen bir iki cümle.
    summary              text CHECK (char_length(summary) <= 500),
    description          text,
    -- Tüm platformlardaki en erken çıkış; ayrıntı game_releases tablosundadır.
    release_date         date,
    release_precision    date_precision NOT NULL DEFAULT 'tba',
    cover_url            text,
    backdrop_url         text,
    logo_url             text,
    -- Ana hikâyeyi bitirme süresi.
    time_to_beat_minutes integer CHECK (time_to_beat_minutes > 0),
    -- Yalnızca gösterilir, sorgulanmaz: { "minimum": {...}, "recommended": {...} }.
    pc_requirements      jsonb,
    status               publish_status NOT NULL DEFAULT 'draft',
    created_at           timestamptz NOT NULL DEFAULT now(),
    updated_at           timestamptz NOT NULL DEFAULT now(),
    CHECK (parent_id IS DISTINCT FROM id),
    CHECK ((release_date IS NULL) = (release_precision = 'tba'))
);
CREATE INDEX games_title_trgm_idx ON games USING gin (title gin_trgm_ops);
-- Listeler yalnızca yayındaki oyunları, çoğunlukla tarihe göre sıralı gösterir.
CREATE INDEX games_published_release_idx ON games (release_date DESC NULLS LAST) WHERE status = 'published';
CREATE INDEX games_parent_id_idx ON games (parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX games_series_id_idx ON games (series_id) WHERE series_id IS NOT NULL;

-- Slug değişince eskisi buraya yazılır; eski adres yenisine yönlendirilir.
CREATE TABLE game_slug_history (
    slug       slug PRIMARY KEY,
    game_id    uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Diğer adlar ve yerel adlar; aramada kullanılır.
CREATE TABLE game_alt_titles (
    game_id uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    title   text NOT NULL CHECK (char_length(title) BETWEEN 1 AND 300),
    PRIMARY KEY (game_id, title)
);
CREATE INDEX game_alt_titles_trgm_idx ON game_alt_titles USING gin (title gin_trgm_ops);

CREATE TABLE platforms (
    id       smallint PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    slug     slug NOT NULL UNIQUE,
    name     text NOT NULL,
    -- Süzgeçlerde gruplamak için: pc, playstation, xbox, nintendo, mobile.
    family   text NOT NULL,
    position smallint NOT NULL DEFAULT 0
);

INSERT INTO platforms (slug, name, family, position) VALUES
    ('pc', 'PC (Windows)', 'pc', 10),
    ('mac', 'macOS', 'pc', 11),
    ('linux', 'Linux', 'pc', 12),
    ('ps5', 'PlayStation 5', 'playstation', 20),
    ('ps4', 'PlayStation 4', 'playstation', 21),
    ('ps3', 'PlayStation 3', 'playstation', 22),
    ('xbox-series', 'Xbox Series X|S', 'xbox', 30),
    ('xbox-one', 'Xbox One', 'xbox', 31),
    ('xbox-360', 'Xbox 360', 'xbox', 32),
    ('switch-2', 'Nintendo Switch 2', 'nintendo', 40),
    ('switch', 'Nintendo Switch', 'nintendo', 41),
    ('wii-u', 'Wii U', 'nintendo', 42),
    ('ios', 'iOS', 'mobile', 50),
    ('android', 'Android', 'mobile', 51);

-- Oyunun bir platformda (ve bölgede) çıkışı. Aynı oyun bir platforma yıllar sonra gelebilir.
CREATE TABLE game_releases (
    game_id           uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    platform_id       smallint NOT NULL REFERENCES platforms (id),
    -- 'worldwide' ya da ISO 3166-1 alfa-2 ülke kodu (küçük harf).
    region            text NOT NULL DEFAULT 'worldwide' CHECK (region ~ '^(worldwide|[a-z]{2})$'),
    release_date      date,
    release_precision date_precision NOT NULL DEFAULT 'tba',
    PRIMARY KEY (game_id, platform_id, region),
    CHECK ((release_date IS NULL) = (release_precision = 'tba'))
);
-- "Bu platformdaki oyunlar" sorgusu için.
CREATE INDEX game_releases_platform_idx ON game_releases (platform_id, game_id);

-- Tür, tema, oyun modu ve bakış açısı tek tabloda; aynı ad farklı türde yeniden kullanılabilir.
CREATE TABLE tags (
    id   integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    kind tag_kind NOT NULL,
    slug slug NOT NULL,
    name text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 80),
    UNIQUE (kind, slug)
);

CREATE TABLE game_tags (
    game_id uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    tag_id  integer NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, tag_id)
);
CREATE INDEX game_tags_tag_idx ON game_tags (tag_id, game_id);

CREATE TABLE companies (
    id          uuid PRIMARY KEY DEFAULT uuidv7(),
    slug        slug NOT NULL UNIQUE,
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 200),
    description text,
    logo_url    text,
    created_at  timestamptz NOT NULL DEFAULT now()
);

-- Rol ilişkidedir: aynı şirket bir oyunun hem geliştiricisi hem yayıncısı olabilir.
CREATE TABLE game_companies (
    game_id    uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    company_id uuid NOT NULL REFERENCES companies (id) ON DELETE CASCADE,
    role       company_role NOT NULL,
    PRIMARY KEY (game_id, company_id, role)
);
CREATE INDEX game_companies_company_idx ON game_companies (company_id, game_id);

-- Dış kaynaklardaki kimlikler (steam appid, wikidata QID...). Birincil anahtar aynı dış kaydın
-- iki oyuna bağlanmasını engeller; içe aktarma eşleştirmeyi buradan yapar.
CREATE TABLE game_external_ids (
    source      text NOT NULL CHECK (source ~ '^[a-z][a-z0-9_]{1,30}$'),
    external_id text NOT NULL CHECK (char_length(external_id) BETWEEN 1 AND 200),
    game_id     uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    fetched_at  timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (source, external_id)
);
CREATE INDEX game_external_ids_game_idx ON game_external_ids (game_id);

-- Ekran görüntüsü ve görsellerde `url` görselin adresidir; fragmanda sağlayıcıdaki video kimliği
-- `video_id` alanındadır (örn. provider = 'youtube').
CREATE TABLE game_media (
    id       uuid PRIMARY KEY DEFAULT uuidv7(),
    game_id  uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    kind     media_kind NOT NULL,
    url      text,
    provider text,
    video_id text,
    position integer NOT NULL DEFAULT 0,
    CHECK ((kind = 'trailer') = (provider IS NOT NULL AND video_id IS NOT NULL)),
    CHECK ((kind <> 'trailer') = (url IS NOT NULL))
);
CREATE INDEX game_media_game_idx ON game_media (game_id, kind, position);

-- Dil desteği: dil kodu BCP 47 (örn. 'en', 'tr', 'pt-BR').
CREATE TABLE game_languages (
    game_id   uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    language  text NOT NULL CHECK (language ~ '^[a-z]{2,3}(-[A-Za-z0-9]{2,8})*$'),
    interface boolean NOT NULL DEFAULT false,
    audio     boolean NOT NULL DEFAULT false,
    subtitles boolean NOT NULL DEFAULT false,
    PRIMARY KEY (game_id, language)
);
-- "Türkçe desteği olan oyunlar" süzgeci için.
CREATE INDEX game_languages_language_idx ON game_languages (language, game_id);

-- Yaş sınırı kurul başına tutulur (pegi: '18', esrb: 'M').
CREATE TABLE game_age_ratings (
    game_id uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    board   text NOT NULL CHECK (board ~ '^[a-z]{2,10}$'),
    rating  text NOT NULL CHECK (char_length(rating) BETWEEN 1 AND 20),
    PRIMARY KEY (game_id, board)
);
