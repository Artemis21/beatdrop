import { WrongGuess, SkippedGuess, NewGuess, EmptyGuess } from "./Guess";

export function Guesses({ guesses }) {
    return <div className="guesses">
        { [0, 1, 2, 3, 4, 5].map(n => {
            if (n < guesses.length) {
                const guess = guesses[n];
                if (guess !== null) {
                    return <WrongGuess guess={guess} key={n} />;
                } else {
                    return <SkippedGuess key={n} />;
                }
            } else if (n === guesses.length) {
                return <NewGuess key={n} />;
            } else {
                return <EmptyGuess key={n} />;
            }
        }) }
    </div>;
}
