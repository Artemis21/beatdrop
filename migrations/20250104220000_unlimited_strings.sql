-- Remove length limits on all cached music data strings.

ALTER TABLE genre
    ALTER COLUMN title TYPE TEXT,
    ALTER COLUMN picture_url TYPE TEXT;

ALTER TABLE album
    ALTER COLUMN title TYPE TEXT,
    ALTER COLUMN deezer_url TYPE TEXT,
    ALTER COLUMN cover_art_url TYPE TEXT;

ALTER TABLE artist
    ALTER COLUMN title TYPE TEXT,
    ALTER COLUMN deezer_url TYPE TEXT,
    ALTER COLUMN picture_url TYPE TEXT;

ALTER TABLE track
    ALTER COLUMN title TYPE TEXT,
    ALTER COLUMN deezer_url TYPE TEXT,
    ALTER COLUMN preview_url TYPE TEXT;
