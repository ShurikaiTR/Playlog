-- Oyunun tek metin alanı "overview" (summary sütunu): açıklama (description) kaldırılır.
-- Özet 500 karakterle sınırlıydı; tek alan olunca uzun metin de alabilsin diye sınır 4000 olur.
-- Sütun denetiminin adı sürüme göre değişebildiği için tanımına bakarak bulunur.
DO $$
DECLARE
    name text;
BEGIN
    FOR name IN
        SELECT conname FROM pg_constraint
         WHERE conrelid = 'games'::regclass AND contype = 'c'
           AND pg_get_constraintdef(oid) LIKE '%char_length(summary)%'
    LOOP
        EXECUTE format('ALTER TABLE games DROP CONSTRAINT %I', name);
    END LOOP;
END $$;

-- Özeti boş olan oyunlarda açıklama özet olur (sınıra sığacak kadarı).
UPDATE games
   SET summary = left(btrim(description), 4000)
 WHERE (summary IS NULL OR btrim(summary) = '')
   AND description IS NOT NULL
   AND btrim(description) <> '';

ALTER TABLE games DROP COLUMN description;
ALTER TABLE games ADD CONSTRAINT games_summary_len CHECK (char_length(summary) <= 4000);
