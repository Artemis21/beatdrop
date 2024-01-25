import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="card card--with-image card--bad">
            <div className="card__image">
                <img src={track.albumCover} />
            </div>
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
            <SongSearch inputId="new_guess" gameId={gameId} />
            <label htmlFor="new_guess" className="card__sub">
                Start typing a song title or artist for suggestions
            </label>
        </div>
    );
}
