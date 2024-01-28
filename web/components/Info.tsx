import { Link } from "react-router-dom";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";

export function Info() {
    // TODO: style this page
    const daily = <GameType game={{ isDaily: true }} />;
    return (
        <Scrollable>
            <h1>Help!</h1>
            <p>
                <b>Beatdrop</b> is a name-that-tune music guessing game inspired by the
                now-discontinued Heardle.
            </p>
            <ol>
                <li>Start a game - try {daily} for your first time.</li>
                <li>Listen to the music. Does it ring a bell?</li>
                <li>Start typing your guess, then pick a search result.</li>
                <li>Incorrect guesses let you hear more of the track.</li>
                <li>If you have no idea, you can skip your guess.</li>
                <li>
                    Use the buttons at the bottom of the page to jump backwards and
                    forwards.
                </li>
                <li>Try to get the song before your guesses run out! Good luck!</li>
            </ol>
            <p>&copy; AM Vellacott 2024 &bull; Music from Deezer</p>
            <Link to="/">Start playing!</Link>
        </Scrollable>
    );
}
