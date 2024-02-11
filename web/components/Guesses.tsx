import { Game } from "../api";
import { GuessQuery } from "./Game";
import { WrongGuess, SkippedGuess, EmptyGuess, NewGuess } from "./Guess";
import { Scrollable } from "./Scrollable";

export function Guesses({
    game: { id, guesses, timedGuess, constants },
    guessQuery,
    setGuessQuery,
}: {
    game: Game;
    guessQuery: GuessQuery;
    setGuessQuery: (q: GuessQuery) => void;
}) {
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
            guessEls.push(
                <NewGuess
                    key={n}
                    gameId={id}
                    timedGuess={timedGuess}
                    guess={guessQuery}
                    setGuess={setGuessQuery}
                />,
            );
        } else {
            guessEls.push(<EmptyGuess key={n} />);
        }
    }
    return (
        <Scrollable>
            <div className="card_stack">{guessEls}</div>
        </Scrollable>
    );
}
