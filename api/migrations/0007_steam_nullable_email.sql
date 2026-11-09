-- Steam ile giriş: Steam e-posta vermez, bu yüzden Steam ile açılan hesabın e-postası olmayabilir.
-- UNIQUE ve CHECK (email = lower(email)) NULL değerleri kabul eder; birden çok NULL aynı anda bulunabilir.
-- 'steam' değeri oauth_provider enum'unda 0001'den beri vardır.
ALTER TABLE users ALTER COLUMN email DROP NOT NULL;
