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

export function NewGuess({ gameId }: { gameId: number }) {
    const search = <SongSearch gameId={gameId} inputId="new_guess" />;
    return (
        <Card title={search} active>
            <label htmlFor="new_guess" className="search_label">
                Start typing a song title or artist for suggestions.
            </label>
        </Card>
    );
}
