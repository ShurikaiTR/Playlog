-- Sosyal katman, 1. adım: kullanıcı listeleri, puanlar ve incelemeler.

CREATE TYPE list_kind AS ENUM ('played', 'play_later', 'custom');
CREATE TYPE list_visibility AS ENUM ('private', 'public');

-- Kullanıcı başına iki sistem listesi (played, play_later) ilk kullanımda oluşturulur; gerisi özel liste.
CREATE TABLE user_lists (
    id          uuid PRIMARY KEY DEFAULT uuidv7(),
    user_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    kind        list_kind NOT NULL DEFAULT 'custom',
    name        text NOT NULL CHECK (char_length(name) BETWEEN 1 AND 60),
    description text NOT NULL DEFAULT '' CHECK (char_length(description) <= 500),
    visibility  list_visibility NOT NULL DEFAULT 'private',
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX user_lists_user_idx ON user_lists (user_id, created_at);
-- Sistem listelerinden kullanıcı başına yalnızca bir tane olur.
CREATE UNIQUE INDEX user_lists_system_uniq ON user_lists (user_id, kind) WHERE kind <> 'custom';

-- Liste ile oyun arasındaki çoktan çoğa ilişki.
CREATE TABLE user_list_games (
    list_id  uuid NOT NULL REFERENCES user_lists (id) ON DELETE CASCADE,
    game_id  uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    added_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (list_id, game_id)
);
-- "Bu oyunu kimler listeledi?" ve toplam sayımları için.
CREATE INDEX user_list_games_game_idx ON user_list_games (game_id, added_at DESC);

-- 10 üzerinden puan; kullanıcı başına oyun başına bir tane.
CREATE TABLE game_ratings (
    user_id    uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id    uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    score      smallint NOT NULL CHECK (score BETWEEN 1 AND 10),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, game_id)
);
CREATE INDEX game_ratings_game_idx ON game_ratings (game_id);

-- Yazılı inceleme; kullanıcı başına oyun başına bir tane. Puanı game_ratings'ten (aynı kullanıcı) gelir.
CREATE TABLE game_reviews (
    user_id    uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id    uuid NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    body       text NOT NULL CHECK (char_length(body) BETWEEN 10 AND 5000),
    spoiler    boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, game_id)
);
CREATE INDEX game_reviews_game_idx ON game_reviews (game_id, created_at DESC);
