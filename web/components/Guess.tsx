import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="card card--bad">
            <img className="card__thumb" src={track.albumCover} />
            <span className="card__title">{track.title}</span>
            <span className="card__sub">{track.artistName}</span>
        </div>
    );
}

export function SkippedGuess() {
    return (
        <div className="card card--skip">
            <FontAwesomeIcon className="card__thumb" icon={faForward} fixedWidth />
            <span className="card__title">Skipped</span>
        </div>
    );
}

export function EmptyGuess() {
    return (
        <div className="card card--empty">
            <FontAwesomeIcon className="card__thumb" icon={faQuestion} fixedWidth />
            <span className="card__title">-------- -----</span>
            <span className="card__sub">--- -------</span>
        </div>
    );
}

export function NewGuess({ gameId }: { gameId: number }) {
    return (
        <div className="card card--active">
            <SongSearch gameId={gameId} />
            <div className="card__sub card__sub--hint">
                Start typing above for suggestions
            </div>
        </div>
    );
}
