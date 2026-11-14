-- Eski kullanıcı adı üretimi Türkçe harfleri ve boşlukları "_" yapıyordu ("Gökhan Güler" →
-- "g_khan_g_ler"). Yalnızca o eski üretimle oluşmuş adlar (görünen adın eski dönüşümüyle birebir
-- aynı olanlar) görünen addan yeniden üretilir: harfler ASCII'ye çevrilir, boşluk ve noktalama
-- atılır ("gokhanguler"). Alınmış ya da 3 karakterden kısa olan adlara dokunulmaz.
DO $$
DECLARE
    r record;
    fixed text;
BEGIN
    FOR r IN
        SELECT id, display_name FROM users
         WHERE display_name IS NOT NULL
           AND username LIKE '%\_%' ESCAPE '\'
           AND username = left(trim(both '_' from regexp_replace(lower(display_name), '[^a-z0-9]', '_', 'g')), 16)
         ORDER BY created_at
    LOOP
        fixed := left(regexp_replace(
            translate(lower(replace(r.display_name, 'İ', 'i')), 'âàáäãåçèéêëìíîïñòóôöõøùúûüýÿğışčšžłđ', 'aaaaaaceeeeiiiinoooooouuuuyygisczsld'),
            '[^a-z0-9]', '', 'g'), 16);
        IF char_length(fixed) >= 3 AND NOT EXISTS (SELECT 1 FROM users WHERE username = fixed) THEN
            UPDATE users SET username = fixed WHERE id = r.id;
        END IF;
    END LOOP;
END $$;
