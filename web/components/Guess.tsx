import { GuessTiming, Track, useInvalidateGame } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";
import { TrackCard } from "./TrackCard";
import { useTimer } from "../utils";
import { useCallback, useEffect } from "react";

export function WrongGuess({ track }: { track: Track }) {
    return <TrackCard track={track} bad />;
}

export function SkippedGuess() {
    return <Card title="Skipped" icon={faForward} />;
}

export function EmptyGuess() {
    return (
        <Card title="-------- -----" icon={faQuestion}>
            --- -------
        </Card>
    );
}

export function NewGuess({
    gameId,
    timedGuess,
    guessQuery,
    setGuessQuery,
}: {
    gameId: number;
    timedGuess: GuessTiming | null;
    guessQuery: string;
    setGuessQuery: (q: string) => void;
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
    const search = (
        <SongSearch
            gameId={gameId}
            inputId="new_guess"
            query={guessQuery}
            setQuery={setGuessQuery}
        />
    );
    return (
        <Card title={search} progress={fractionElapsed} active>
            <label htmlFor="new_guess" className="search_label">
                <p>Start typing a song title or artist for suggestions.</p>
                <p>
                    {remainingTime && (
                        <p>{(remainingTime / 1000).toFixed(2)}s remaining</p>
                    )}
                </p>
            </label>
        </Card>
    );
}
