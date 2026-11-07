-- Apple ile giriş. Yeni enum değeri aynı işlemde kullanılmadığı için ADD VALUE güvenlidir.
ALTER TYPE oauth_provider ADD VALUE IF NOT EXISTS 'apple';
