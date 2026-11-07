-- E-postayla tek kullanımlık giriş kodu. Yeni enum değeri aynı işlemde kullanılmadığı için ADD VALUE güvenlidir.
ALTER TYPE email_code_purpose ADD VALUE IF NOT EXISTS 'login_code';
