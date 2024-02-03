import { useEffect, useState } from "react";
import { Track, searchTracks, useNewGuess } from "../api";
import { useThrottled, classModifiers } from "../utils";

export function SongSearch({
    gameId,
    inputId,
    query,
    setQuery,
}: {
    gameId: number;
    inputId: string;
    query: string;
    setQuery: (q: string) => void;
}) {
    const debouncedQ = useThrottled(query, 500);
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
    if (!active || query === "" || id !== null) {
        results = null;
    } else if (error) {
        results = <SearchResultsPlaceholder message={error.toString()} />;
    } else if (tracks === undefined || (tracks.length === 0 && query !== debouncedQ)) {
        results = <SearchResultsPlaceholder message="Loading..." />;
    } else if (tracks.length === 0) {
        results = <SearchResultsPlaceholder message="No results found." />;
    } else {
        results = <SearchResults tracks={tracks} setQ={setQuery} setId={setId} />;
    }
    let button;
    if (id === null) {
        button = (
            <GuessButton gameId={gameId} guess={null} afterSubmit={() => setQuery("")} />
        );
    } else {
        button = (
            <GuessButton gameId={gameId} guess={id} afterSubmit={() => setQuery("")} />
        );
    }
    return (
        <div className="form_row">
            <div className="search">
                <input
                    className="input"
                    type="search"
                    placeholder="Title or artist..."
                    onChange={e => {
                        setId(null);
                        setQuery(e.target.value);
                    }}
                    onFocus={() => setActive(true)}
                    onBlur={() => setActive(false)}
                    value={query}
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

function GuessButton({
    gameId,
    guess,
    afterSubmit,
}: {
    gameId: number;
    guess: number | null;
    afterSubmit: () => void;
}) {
    const { mutate, isLoading } = useNewGuess();
    const className = classModifiers("submit", { secondary: guess === null });
    if (isLoading) {
        return <button className={className}>...</button>;
    }
    return (
        <button
            className={className}
            onClick={() => mutate({ gameId, trackId: guess }).then(afterSubmit)}
        >
            {guess === null ? "Skip" : "Guess"}
        </button>
    );
}
