-- Kullanıcı rolleri. Enum sırası karşılaştırmayı da belirler: user < moderator < admin.
CREATE TYPE user_role AS ENUM ('user', 'moderator', 'admin');

-- Sabit varsayılanlı NOT NULL sütun eklemek PostgreSQL 11+ sürümlerde tabloyu yeniden yazmaz;
-- mevcut kullanıcılar anında 'user' olur.
ALTER TABLE users ADD COLUMN role user_role NOT NULL DEFAULT 'user';

-- Yetkili listesi ve "son admin" denetimi için: kullanıcıların çoğu 'user' olduğundan
-- kısmi indeks yalnızca birkaç satır tutar.
CREATE INDEX users_staff_role_idx ON users (role) WHERE role <> 'user';
