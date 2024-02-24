-- Music data cached from Deezer

-- Information on a genre
CREATE TABLE IF NOT EXISTS genre (
    -- The genre ID (from Deezer)
    id INTEGER PRIMARY KEY,

    -- The genre name
    title VARCHAR(255) NOT NULL,

    -- A picture for the genre
    picture_url VARCHAR(255) NOT NULL
);

-- Information on an album
CREATE TABLE IF NOT EXISTS album (
    -- The album ID (from Deezer)
    id INTEGER PRIMARY KEY,

    -- The album name
    title VARCHAR(255) NOT NULL,

    -- A link to the album on Deezer.
    deezer_url VARCHAR(255) NOT NULL,

    -- A link to the album's cover art
    cover_art_url VARCHAR(255) NOT NULL
);

-- Many-to-many genres albums belong to
CREATE TABLE IF NOT EXISTS album_genre (
    -- The album
    album_id INTEGER REFERENCES album(id) NOT NULL,

    -- The genre
    genre_id INTEGER REFERENCES genre(id) NOT NULL,

    PRIMARY KEY (album_id, genre_id)
);

-- Information on an artist
CREATE TABLE IF NOT EXISTS artist (
    -- The artist ID (from Deezer)
    id INTEGER PRIMARY KEY,

    -- The artist name
    title VARCHAR(255) NOT NULL,

    -- A link to the artist on Deezer.
    deezer_url VARCHAR(255) NOT NULL,

    -- A link to the artist's picture
    picture_url VARCHAR(255) NOT NULL
);

-- Information on a track
CREATE TABLE IF NOT EXISTS track (
    -- The track ID (from Deezer)
    id INTEGER PRIMARY KEY,

    -- The track name
    title VARCHAR(255) NOT NULL,

    -- The tracks' ranking on Deezer
    deezer_rank INTEGER NOT NULL,

    -- The artist
    artist_id INTEGER REFERENCES artist(id) NOT NULL,

    -- The album
    album_id INTEGER REFERENCES album(id) NOT NULL,

    -- A link to the track on Deezer.
    deezer_url VARCHAR(255) NOT NULL,

    -- A link to the 30s preview mp3
    preview_url VARCHAR(255) NOT NULL
);

-- Actual game data

-- A user account.
--
-- Currently users are all anonymous, so log in happens by storing a secret created on first visit.
-- In the future other login methods may be added.
CREATE TABLE IF NOT EXISTS account (
    id SERIAL PRIMARY KEY,

    -- The user's display name, if set
    display_name VARCHAR(255) DEFAULT NULL,

    -- The 64 byte SHA512 hash of the 32 byte login token
    secret_hash BYTEA NOT NULL,

    -- When the account was created
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT TIMEZONE('utc', NOW())
);

-- A game, completed or ongoing.
CREATE TABLE IF NOT EXISTS game (
    id SERIAL PRIMARY KEY,

    -- The user who played the game
    account_id INTEGER REFERENCES account(id) NOT NULL ON DELETE CASCADE,

    -- When the game was started
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT TIMEZONE('utc', NOW()),

    -- The game mode. Daily mode is never timed, so these flags are mutually exclusive.
    is_daily BOOLEAN NOT NULL,
    is_timed BOOLEAN NOT NULL,

    -- Genre, if this is a genre-specific game mode
    genre_id INTEGER REFERENCES genre(id),

    -- The track the user guesses
    track_id INTEGER REFERENCES track(id) NOT NULL,

    -- Did the user win (null for ongoing games)
    won BOOLEAN DEFAULT NULL
);

-- A guess in a game
CREATE TABLE IF NOT EXISTS game_guess (
    -- The game this guess is for
    game_id INTEGER REFERENCES game(id) NOT NULL ON DELETE CASCADE,

    -- Which number guess this was (1 for the first guess, etc.)
    guess_number INTEGER NOT NULL,

    PRIMARY KEY (game_id, guess_number),

    -- The track the user guessed, or null if they skipped this guess
    track_id INTEGER REFERENCES track(id),

    -- The time the guess was made
    guessed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT TIMEZONE('utc', NOW())
);

-- A record of the track of the day each day
CREATE TABLE IF NOT EXISTS daily_track (
    -- Which date this was the track of the day
    for_day DATE PRIMARY KEY DEFAULT TIMEZONE('utc', NOW())::DATE,

    -- The track of the day
    track_id INTEGER REFERENCES track(id) NOT NULL
);
