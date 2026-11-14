-- Listenin kapağı, listedeki oyunlardan biridir (görsel yüklemesi yok). NULL: listedeki ilk oyun.
-- Oyun silinirse ya da listeden çıkarılırsa kapak varsayılana döner.
ALTER TABLE user_lists
    ADD COLUMN cover_game_id uuid REFERENCES games (id) ON DELETE SET NULL;
