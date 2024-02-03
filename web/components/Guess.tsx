import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";
import { TrackCard } from "./TrackCard";

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
    guessQuery,
    setGuessQuery,
}: {
    gameId: number;
    guessQuery: string;
    setGuessQuery: (q: string) => void;
}) {
    const search = (
        <SongSearch
            gameId={gameId}
            inputId="new_guess"
            query={guessQuery}
            setQuery={setGuessQuery}
        />
    );
    return (
        <Card title={search} active>
            <label htmlFor="new_guess" className="search_label">
                Start typing a song title or artist for suggestions.
            </label>
        </Card>
    );
}
