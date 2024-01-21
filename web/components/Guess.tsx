import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="card card--with-image card--bad">
            <img className="card__image" src={track.albumCover} />
            <span className="card__title">{track.title}</span>
            <span className="card__sub">{track.artistName}</span>
        </div>
    );
}

export function SkippedGuess() {
    return (
        <div className="card card--no-sub">
            <FontAwesomeIcon className="card__icon" icon={faForward} />
            <span className="card__title">Skipped</span>
        </div>
    );
}

export function EmptyGuess() {
    return (
        <div className="card card--with-image">
            <FontAwesomeIcon className="card__icon" icon={faQuestion} />
            <span className="card__title">-------- -----</span>
            <span className="card__sub">--- -------</span>
        </div>
    );
}

export function NewGuess({ gameId }: { gameId: number }) {
    return (
        <div className="card card--active-guess">
            <SongSearch gameId={gameId} />
            <div className="card__sub">
                Start typing above for suggestions
            </div>
        </div>
    );
}
