import { SongSearch } from "./SongSearch";

export function WrongGuess({ guess }) {
    return <div className="guess guess--wrong">
        <img className="guess__thumb" src={guess.cover} />
        <span className="guess__title">{guess.title}</span>
        <span className="guess__sub">{guess.artist}</span>
    </div>;
}

export function SkippedGuess() {
    return <div className="guess guess--skip">
        <i className="guess__thumb fa-solid fa-fw fa-forward"></i>
        <span className="guess__title">Skipped</span>
    </div>;
}

export function EmptyGuess() {
    return <div className="guess guess--empty">
        <i className="guess__thumb fa-solid fa-fw fa-question"></i>
        <span className="guess__title">-------- -----</span>
        <span className="guess__sub">--- -------</span>
    </div>;
}

export function NewGuess() {
    return <div className="guess guess--active">
        <SongSearch className="guess__title" />
        <span className="guess__sub"></span>
    </div>;
}
