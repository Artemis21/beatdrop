import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Track } from "../api";
import { SongSearch } from "./SongSearch";
import { faForward, faQuestion } from "@fortawesome/free-solid-svg-icons";

export function WrongGuess({ track }: { track: Track }) {
    return (
        <div className="stack__item stack__item--bad">
            <img className="stack__item__thumb" src={track.albumCover} />
            <span className="stack__item__title">{track.title}</span>
            <span className="stack__item__sub">{track.artistName}</span>
        </div>
    );
}

export function SkippedGuess() {
    return (
        <div className="stack__item stack__item--skip">
            <FontAwesomeIcon className="stack__item__thumb" icon={faForward} fixedWidth />
            <span className="stack__item__title">Skipped</span>
        </div>
    );
}

export function EmptyGuess() {
    return (
        <div className="stack__item stack__item--empty">
            <FontAwesomeIcon
                className="stack__item__thumb"
                icon={faQuestion}
                fixedWidth
            />
            <span className="stack__item__title">-------- -----</span>
            <span className="stack__item__sub">--- -------</span>
        </div>
    );
}

export function NewGuess() {
    return (
        <div className="stack__item stack__item--active">
            <SongSearch />
            <div className="stack__item__sub stack__item__sub--hint">
                Start typing above for suggestions
            </div>
        </div>
    );
}
