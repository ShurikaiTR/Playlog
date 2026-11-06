-- Aşama 1: giriş (kimlik doğrulama) tabloları.
-- PostgreSQL 18+ gerekir: uuidv7() yerleşik fonksiyondur.
--
-- Kullanıcı adı ve e-posta uygulama tarafında küçük harfe çevrilip kaydedilir;
-- CHECK kısıtı bunu veritabanında da zorlar. Böylece düz UNIQUE yeterli olur
-- (lower() ifadeli indeks ya da citext gerekmez, B-tree tekilleştirme çalışır).

CREATE TYPE oauth_provider AS ENUM ('google', 'discord', 'steam');
CREATE TYPE email_code_purpose AS ENUM ('signup', 'password_reset');

CREATE TABLE users (
    id                uuid PRIMARY KEY DEFAULT uuidv7(),
    username          text NOT NULL UNIQUE CHECK (username ~ '^[a-z0-9_]{3,20}$'),
    email             text NOT NULL UNIQUE CHECK (email = lower(email)),
    email_verified_at timestamptz,
    avatar_url        text,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now()
);

-- E-posta + şifre ile giriş. Google/Discord ile açılan hesapta satır olmaz.
CREATE TABLE credentials (
    user_id       uuid PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,
    password_hash text NOT NULL,
    updated_at    timestamptz NOT NULL DEFAULT now()
);

-- Google, Discord ve ileride Steam bağları.
-- UNIQUE (user_id, provider) aynı zamanda user_id yabancı anahtarını indeksler.
CREATE TABLE oauth_accounts (
    id                  uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id             uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider            oauth_provider NOT NULL,
    provider_account_id text NOT NULL,
    created_at          timestamptz NOT NULL DEFAULT now(),
    UNIQUE (provider, provider_account_id),
    UNIQUE (user_id, provider)
);

CREATE TABLE sessions (
    id           uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id      uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token_hash   bytea NOT NULL UNIQUE, -- çerezdeki tokenin SHA-256 özeti
    user_agent   text,
    ip           inet,
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    expires_at   timestamptz NOT NULL
);
CREATE INDEX sessions_user_id_idx    ON sessions (user_id);
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at); -- süresi dolanları temizlemek için

-- 6 haneli kayıt kodu ve şifre sıfırlama kodu.
CREATE TABLE email_codes (
    id          uuid PRIMARY KEY DEFAULT uuidv7(),
    purpose     email_code_purpose NOT NULL,
    email       text NOT NULL CHECK (email = lower(email)),
    code_hash   bytea NOT NULL,
    pending     jsonb, -- kayıtta: kullanıcı adı + şifre özeti
    attempts    smallint NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    created_at  timestamptz NOT NULL DEFAULT now(),
    expires_at  timestamptz NOT NULL,
    consumed_at timestamptz,
    CHECK (expires_at > created_at)
);
-- Yalnızca kullanılmamış kodlar aranır: kısmi indeks küçük ve hızlı kalır.
CREATE INDEX email_codes_active_idx ON email_codes (email, purpose) WHERE consumed_at IS NULL;
