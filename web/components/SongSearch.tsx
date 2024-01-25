import { useEffect, useState } from "react";
import { Track, searchTracks, useNewGuess } from "../api";
import { useThrottled } from "../utils";

export function SongSearch({ gameId, inputId }: { gameId: number, inputId: string }) {
    const [q, setQ] = useState("");
    const debouncedQ = useThrottled(q, 500);
    const [id, setId] = useState<number | null>(null);
    const [active, setActive] = useState(false);
    const [tracks, setTracks] = useState<Track[] | undefined>(undefined);
    const [error, setError] = useState<object | null>(null);
    useEffect(() => {
        if (id === null && debouncedQ !== "") {
            let cancelled = false;
            searchTracks(debouncedQ)
                .then(data => cancelled || setTracks(data.tracks))
                .catch(error => cancelled || setError(error));
            return () => {
                cancelled = true;
            };
        }
    }, [debouncedQ, id]);
    let results;
    if (!active || q === "" || id !== null) {
        results = null;
    } else if (error) {
        results = <SearchResultsPlaceholder message={error.toString()} />;
    } else if (tracks === undefined || (tracks.length === 0 && q !== debouncedQ)) {
        results = <SearchResultsPlaceholder message="Loading..." />;
    } else if (tracks.length === 0) {
        results = <SearchResultsPlaceholder message="No results found." />;
    } else {
        results = <SearchResults tracks={tracks} setQ={setQ} setId={setId} />;
    }
    let button;
    if (id === null) {
        button = <GuessButton gameId={gameId} guess={null} />;
    } else {
        button = <GuessButton gameId={gameId} guess={id} />;
    }
    return (
        <div className="card__title">
            <div className="search">
                <input
                    className="search__input"
                    type="search"
                    placeholder="Title or artist..."
                    onChange={e => {
                        setId(null);
                        setQ(e.target.value);
                    }}
                    onFocus={() => setActive(true)}
                    onBlur={() => setActive(false)}
                    value={q}
                    id={inputId}
                />
                {results}
            </div>
            {button}
        </div>
    );
}

function SearchResults({
    tracks,
    setQ,
    setId,
}: {
    tracks: Track[];
    setQ: (q: string) => void;
    setId: (id: number) => void;
}) {
    return (
        <div className="search__results">
            {tracks.map(track => {
                const displayName = `${track.title} - ${track.artistName}`;
                const click = () => {
                    setId(track.id);
                    setQ(displayName);
                };
                return (
                    <button
                        className="search__results__result"
                        key={track.id}
                        onMouseDown={click}
                    >
                        {displayName}
                    </button>
                );
            })}
        </div>
    );
}

function SearchResultsPlaceholder({ message }: { message: string }) {
    return (
        <div className="search__results">
            <div className="search__results__placeholder">{message}</div>
        </div>
    );
}

function GuessButton({ gameId, guess }: { gameId: number; guess: number | null }) {
    const { mutate, isLoading } = useNewGuess();
    const kind = guess === null ? "skip" : "guess";
    if (isLoading) {
        return (
            <button
                className={`guess_button guess_button--${kind} guess_button--loading`}
            >
                ...
            </button>
        );
    }
    return (
        <button
            className={`guess_button guess_button--${kind}`}
            onClick={() => mutate({ gameId, trackId: guess })}
        >
            {guess === null ? "Skip" : "Guess"}
        </button>
    );
}
