import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="card card--bad">
            <div className="card__image">
                {/* FIXME: pick size appropriately (small/medium/big/xl) */}
                <img src={`${track.albumCover}?size=xl`} />
            </div>
            <span className="card__title">{track.title}</span>
            <span className="card__sub">{track.artistName}</span>
        </div>
    );
}

export function SkippedGuess() {
    return (
        <div className="card">
            <FontAwesomeIcon className="card__icon" icon={faForward} />
            <span className="card__title">Skipped</span>
        </div>
    );
}

export function EmptyGuess() {
    return (
        <div className="card">
            <FontAwesomeIcon className="card__icon" icon={faQuestion} />
            <span className="card__title">-------- -----</span>
            <span className="card__sub">--- -------</span>
        </div>
    );
}

export function NewGuess({ gameId }: { gameId: number }) {
    return (
        <div className="card card--active">
            <SongSearch inputId="new_guess" gameId={gameId} />
            <p className="card__sub">
                <label htmlFor="new_guess" className="search_label">
                    Start typing a song title or artist for suggestions.
                </label>
            </p>
        </div>
    );
}
