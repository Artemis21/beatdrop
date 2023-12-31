import { Game } from "../fetcher";
import { WrongGuess, SkippedGuess, NewGuess, EmptyGuess } from "./Guess";

export function Guesses({ game }: { game: Game }) {
    const { guesses, constants } = game;
    const guessEls = [];
    for (let n = 0; n < constants.maxGuesses; n++) {
        if (n < guesses.length) {
            const guess = guesses[n];
            if (guess !== null ) {
                guessEls.push(<WrongGuess track={guess.track} key={n} />);
            } else {
                guessEls.push(<SkippedGuess key={n} />);
            }
        } else if (n === guesses.length) {
            guessEls.push(<NewGuess key={n} />);
        } else {
            guessEls.push(<EmptyGuess key={n} />);
        }
    }
    return <div className="guesses">{ guessEls }</div>;
}
