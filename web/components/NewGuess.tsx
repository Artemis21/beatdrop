import { RefObject, useEffect, useRef, useState } from "react";
import { Track, searchTracks, useNewGuess } from "../api";
import { useThrottled, classModifiers } from "../utils";
import { GuessQuery } from "./Game";
import { faSearch } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Card } from "./Card";
import { Resign } from "./Resign";

export function NewGuess({
    gameId,
    guess,
    setGuess,
}: {
    gameId: number;
    guess: GuessQuery;
    setGuess: (q: GuessQuery) => void;
}) {
    const [active, setActive] = useState(false);
    const searchEl = useRef<HTMLDivElement>(null);
    const inputEl = useRef<HTMLInputElement>(null);
    return (
        <Card active>
            <form className="form_col" onSubmit={e => e.preventDefault()}>
                <div
                    className="search"
                    ref={searchEl}
                    onFocus={() => setActive(true)}
                    onBlur={e => {
                        // check focus actually moved outside of this element
                        if (!searchEl.current?.contains(e.relatedTarget)) {
                            setActive(false);
                        }
                    }}
                >
                    <SearchInput
                        inputRef={inputEl}
                        value={guess.query}
                        setValue={q => setGuess({ query: q, id: null })}
                    />
                    {active && guess.id === null && (
                        <SearchResults
                            query={guess.query}
                            setGuess={guess => {
                                setGuess(guess);
                                inputEl.current?.focus();
                            }}
                        />
                    )}
                </div>
                <div className="form_row">
                    <Resign gameId={gameId} />
                    <GuessButton gameId={gameId} guess={guess} setGuess={setGuess} />
                </div>
            </form>
        </Card>
    );
}

function SearchInput({
    inputRef,
    value,
    setValue,
}: {
    inputRef: RefObject<HTMLInputElement>;
    value: string;
    setValue: (_: string) => void;
}) {
    return (
        <>
            <label htmlFor="song_search_input">
                <FontAwesomeIcon
                    icon={faSearch}
                    className="search__icon"
                    title="Search for a title or artist"
                />
            </label>
            <input
                id="song_search_input"
                className="search__input"
                type="search"
                placeholder="Title or artist..."
                ref={inputRef}
                onChange={e => setValue(e.target.value)}
                value={value}
                autoFocus
            />
        </>
    );
}

function SearchResults({
    query,
    setGuess,
}: {
    query: string;
    setGuess: (_: GuessQuery) => void;
}) {
    const debouncedQ = useThrottled(query, 500);
    const [tracks, setTracks] = useState<Track[] | undefined>(undefined);
    const [error, setError] = useState<object | null>(null);
    useEffect(() => {
        if (debouncedQ !== "") {
            let cancelled = false;
            searchTracks(debouncedQ)
                .then(data => cancelled || setTracks(data.tracks))
                .catch(error => cancelled || setError(error));
            return () => {
                cancelled = true;
            };
        }
    }, [debouncedQ]);
    let placeholder = null;
    if (query === "") {
        return null;
    } else if (error) {
        placeholder = error.toString();
    } else if (tracks === undefined || (tracks.length === 0 && query !== debouncedQ)) {
        placeholder = "Loading...";
    } else if (tracks.length === 0) {
        placeholder = "No results found.";
    }
    return (
        <div className="search__results">
            {placeholder && (
                <div className="search__results__placeholder">{placeholder}</div>
            )}
            {tracks?.map(track => {
                const displayName = `${track.title} - ${track.artistName}`;
                const click = () => setGuess({ query: displayName, id: track.id });
                return (
                    <button
                        className="search__results__result"
                        key={track.id}
                        onClick={click}
                        type="button"
                    >
                        {displayName}
                    </button>
                );
            })}
        </div>
    );
}

function GuessButton({
    gameId,
    guess,
    setGuess,
}: {
    gameId: number;
    guess: GuessQuery;
    setGuess: (_: GuessQuery) => void;
}) {
    const { mutate, isLoading } = useNewGuess();
    const className = classModifiers("submit", { secondary: guess.id === null });
    if (isLoading) {
        return <button className={className}>...</button>;
    }
    return (
        <button
            className={className}
            onClick={async () => {
                await mutate({ gameId, trackId: guess.id });
                setGuess({ query: "", id: null });
            }}
        >
            {guess.id === null ? "Skip" : "Guess"}
        </button>
    );
}

// May be needed for implementing timed games:
/*
export function NewGuess({
    gameId,
    timedGuess,
    guess,
    setGuess,
}: {
    gameId: number;
    timedGuess: GuessTiming | null;
    guess: GuessQuery;
    setGuess: (q: GuessQuery) => void;
}) {
    const time = useTimer(20);
    // We have to `useCallback` because otherwise `invalidateGame` is a new
    // function every render, and the `useEffect` below will run every 20ms.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    const invalidateGame = useCallback(useInvalidateGame(gameId), [gameId]);
    let remainingTime: number | null, fractionElapsed;
    if (timedGuess) {
        const startedAt = new Date(timedGuess.startedAt).getTime();
        const elapsed = time - startedAt;
        remainingTime = Math.max(0, timedGuess.lengthMillis - elapsed);
        fractionElapsed = elapsed / timedGuess.lengthMillis;
    } else {
        remainingTime = null;
        fractionElapsed = 0;
    }
    useEffect(() => {
        if (!timedGuess) return;
        const elapsed = Date.now() - new Date(timedGuess.startedAt).getTime();
        const timeout = setTimeout(invalidateGame, timedGuess.lengthMillis - elapsed);
        return () => clearTimeout(timeout);
    }, [timedGuess, invalidateGame]);
    const search = <NewGuess gameId={gameId} guess={guess} setGuess={setGuess} />;
    return (
        <Card title={search} progress={fractionElapsed} active>
            {remainingTime && <p>{(remainingTime / 1000).toFixed(2)}s remaining</p>}
        </Card>
    );
}
*/
