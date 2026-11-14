-- İçe aktarma: gözden geçirilecek kayıtlar ve elle düzenlenip kilitlenen alanlar.
--
-- review_reason: kaynaklardan gelen veri güvenilmezse (tarih yok, iki kaynak çelişiyor) nedeni;
--   NULL ise kayıt sorunsuzdur. Yönetici oyunu kaydedince temizlenir.
-- locked_fields: yönetici panelinde elle değiştirilen alan grupları; sonraki içe aktarmalar
--   bunların üzerine yazmaz.
ALTER TABLE games
    ADD COLUMN review_reason text CHECK (char_length(review_reason) <= 200),
    ADD COLUMN locked_fields text[] NOT NULL DEFAULT '{}';

CREATE INDEX games_review_idx ON games (updated_at DESC) WHERE review_reason IS NOT NULL;
