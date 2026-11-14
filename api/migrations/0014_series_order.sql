-- Seri içindeki önerilen oynama sırası: yönetici elle belirler.
--
-- series_position: serideki sıra (0'dan başlar); NULL ise sıra verilmemiştir. Bir serinin hiçbir
--   oyununda sıra yoksa seri çıkış tarihine göre gösterilir.
-- series_note: sıradaki oyunun yanında görünen kısa not ("önce bunu oyna").
-- DLC ve ek paketler seri sırasına girmez; onlar parent_id ile ana oyuna bağlanır.
ALTER TABLE games
    ADD COLUMN series_position integer CHECK (series_position >= 0),
    ADD COLUMN series_note text CHECK (char_length(series_note) <= 200),
    ADD CONSTRAINT games_series_order_needs_series
        CHECK (series_id IS NOT NULL OR (series_position IS NULL AND series_note IS NULL));
