import { Track } from "../api";
import { Icon } from "./Icon";
import { SongSearch } from "./SongSearch";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="guess guess--wrong">
            <img className="guess__thumb" src={track.albumCover} />
            <span className="guess__title">{track.title}</span>
            <span className="guess__sub">{track.artistName}</span>
        </div>
    );
}

export function SkippedGuess() {
    return (
        <div className="guess guess--skip">
            <Icon className="guess__thumb" icon="forward" />
            <span className="guess__title">Skipped</span>
        </div>
    );
}

export function EmptyGuess() {
    return (
        <div className="guess guess--empty">
            <Icon className="guess__thumb" icon="question" />
            <span className="guess__title">-------- -----</span>
            <span className="guess__sub">--- -------</span>
        </div>
    );
}

export function NewGuess() {
    return (
        <div className="guess guess--active">
            <SongSearch />
            <div className="guess__sub guess__sub--hint">
                Start typing above for suggestions
            </div>
        </div>
    );
}
