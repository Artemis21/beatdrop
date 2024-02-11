import { useEffect, useState } from "react";
import { Track, searchTracks, useNewGuess } from "../api";
import { useThrottled, classModifiers } from "../utils";
import { GuessQuery } from "./Game";

export function SongSearch({
    gameId,
    inputId,
    guess,
    setGuess,
}: {
    gameId: number;
    inputId: string;
    guess: GuessQuery;
    setGuess: (q: GuessQuery) => void;
}) {
    const debouncedQ = useThrottled(guess.query, 500);
    const [active, setActive] = useState(false);
    const [tracks, setTracks] = useState<Track[] | undefined>(undefined);
    const [error, setError] = useState<object | null>(null);
    useEffect(() => {
        if (guess.id === null && debouncedQ !== "") {
            let cancelled = false;
            searchTracks(debouncedQ)
                .then(data => cancelled || setTracks(data.tracks))
                .catch(error => cancelled || setError(error));
            return () => {
                cancelled = true;
            };
        }
    }, [debouncedQ, guess]);
    let results;
    if (!active || guess.query === "" || guess.id !== null) {
        results = null;
    } else if (error) {
        results = <SearchResultsPlaceholder message={error.toString()} />;
    } else if (
        tracks === undefined ||
        (tracks.length === 0 && guess.query !== debouncedQ)
    ) {
        results = <SearchResultsPlaceholder message="Loading..." />;
    } else if (tracks.length === 0) {
        results = <SearchResultsPlaceholder message="No results found." />;
    } else {
        results = <SearchResults tracks={tracks} setGuess={setGuess} />;
    }
    let button;
    const resetGuess = () => setGuess({ query: "", id: null });
    if (guess.id === null) {
        button = <GuessButton gameId={gameId} guess={null} afterSubmit={resetGuess} />;
    } else {
        button = (
            <GuessButton gameId={gameId} guess={guess.id} afterSubmit={resetGuess} />
        );
    }
    return (
        <div className="form_row">
            <div className="search">
                <input
                    className="input"
                    type="search"
                    placeholder="Title or artist..."
                    onChange={e =>
                        setGuess({
                            query: e.target.value,
                            id: null,
                        })
                    }
                    onFocus={() => setActive(true)}
                    onBlur={() => setActive(false)}
                    value={guess.query}
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
    setGuess,
}: {
    tracks: Track[];
    setGuess: (q: GuessQuery) => void;
}) {
    return (
        <div className="search__results">
            {tracks.map(track => {
                const displayName = `${track.title} - ${track.artistName}`;
                const click = () => setGuess({ query: displayName, id: track.id });
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
