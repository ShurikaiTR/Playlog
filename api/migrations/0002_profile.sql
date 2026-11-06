-- Profil alanları: görünen ad ve pazarlama e-postası izni.
ALTER TABLE users
    ADD COLUMN display_name text CHECK (char_length(display_name) BETWEEN 1 AND 40),
    -- İzin verildiği an kaydedilir (NULL = izin yok); ispat ve geri çekme için.
    ADD COLUMN marketing_opt_in_at timestamptz;
