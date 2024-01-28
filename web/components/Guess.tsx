import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";

export function WrongGuess({ track }: { track: Track }) {
    const image = {
        // FIXME: pick size appropriately (small/medium/big/xl)
        src: `${track.albumCover}?size=xl`,
        alt: `Album cover for ${track.albumTitle}`,
    };
    return (
        <Card title={track.title} image={image} bad>
            {track.artistName}
        </Card>
    );
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
