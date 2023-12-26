import { WrongGuess, SkippedGuess, EmptyGuess } from "./Guess";

export function Guesses({ guesses }) {
    return <div className="guesses">
        { [1, 2, 3, 4, 5, 6].map(n => {
            if (n < guesses.length) {
                const guess = guesses[n];
                if (guess !== null) {
                    return WrongGuess(guess);
                } else {
                    return SkippedGuess();
                }
            } else if (n === guesses.length) {
                return NewGuess();
            } else {
                return EmptyGuess();
            }
        }) }
    </div>;
}
