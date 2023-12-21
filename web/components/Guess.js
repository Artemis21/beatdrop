import { SongSearch } from "./SongSearch";

export function WrongGuess({ guess }) {
    return <div class="guess guess--wrong">
        <img class="guess__thumb" src={guess.cover} />
        <span class="guess__title">{guess.title}</span>
        <span class="guess__sub">{guess.artist}</span>
    </div>;
}

export function SkippedGuess() {
    return <div class="guess guess--skip">
        <i class="guess__thumb fa-solid fa-fw fa-forward"></i>
        <span class="guess__title">Skipped</span>
    </div>;
}

export function EmptyGuess() {
    return <div class="guess guess--empty">
        <i class="guess__thumb fa-solid fa-fw fa-question"></i>
        <span class="guess__title">-------- -----</span>
        <span class="guess__sub">--- -------</span>
    </div>;
}

export function NewGuess() {
    return <div class="guess guess--active">
        <SongSearch class="guess__title" />
        <span class="guess__sub"></span>
    </div>;
}
