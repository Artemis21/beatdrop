import { Game } from "../api";
import { WrongGuess, SkippedGuess, EmptyGuess, NewGuess } from "./Guess";

export function Guesses({ game: { id, guesses, constants } }: { game: Game }) {
    const guessEls = [];
    for (let n = 0; n < constants.maxGuesses; n++) {
        if (n < guesses.length) {
            const guess = guesses[n].track;
            if (guess !== null) {
                guessEls.push(<WrongGuess track={guess} key={n} />);
            } else {
                guessEls.push(<SkippedGuess key={n} />);
            }
        } else if (n === guesses.length) {
            guessEls.push(<NewGuess key={n} gameId={id} />);
        } else {
            guessEls.push(<EmptyGuess key={n} />);
        }
    }
    return <div className="stack">{guessEls}</div>;
}
