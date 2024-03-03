import { Track } from "../api";
import {
    fa0,
    fa1,
    fa2,
    fa3,
    fa4,
    fa5,
    fa6,
    fa7,
    fa8,
    fa9,
    faForward,
    faQuestion,
} from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";
import { TrackCard } from "./TrackCard";
import { Game, useResignGame } from "../api";
import { classModifiers } from "../utils";
import { GuessQuery } from "./Game";
import { NewGuess } from "./NewGuess";
import { Scrollable } from "./Scrollable";

export function Guesses({
    game: { id, guesses, constants },
    guessQuery,
    setGuessQuery,
}: {
    game: Game;
    guessQuery: GuessQuery;
    setGuessQuery: (q: GuessQuery) => void;
}) {
    const guessEls = [];
    let i = 0;
    while (i < guesses.length) {
        const start = i;
        let guess = guesses[i];
        let skipped = 0;
        while (guess && guess.track === null) {
            skipped++;
            guess = guesses[++i];
        }
        if (skipped > 0) {
            guessEls.push(<SkippedGuesses key={start} count={skipped} />);
        }
        if (guess) {
            guessEls.push(<WrongGuess track={guess.track} key={i} />);
        }
        i++;
    }
    return (
        <Scrollable>
            <div className="card_stack">
                {guessEls}
                <NewGuess gameId={id} guess={guessQuery} setGuess={setGuessQuery} />
                <RemainingGuesses remaining={constants.maxGuesses - guesses.length - 1} />
                <ResignButton gameId={id} />
            </div>
        </Scrollable>
    );
}

function ResignButton({ gameId }: { gameId: number }) {
    const { mutate, isLoading } = useResignGame();
    const className = classModifiers("link_button", { danger: true });
    if (isLoading) return <button className={className}>...</button>;
    const onClick = async () => {
        if (confirm("Are you sure you want to resign this game?")) {
            await mutate({ gameId });
        }
    };
    return (
        <button className={className} onClick={onClick}>
            Give Up
        </button>
    );
}

function WrongGuess({ track }: { track: Track }) {
    return <TrackCard track={track} bad />;
}

function SkippedGuesses({ count }: { count: number }) {
    let title;
    if (count === 1) title = "Skipped";
    else title = `Skipped ${count} guesses`;
    return <Card title={title} icon={faForward} />;
}

function RemainingGuesses({ remaining }: { remaining: number }) {
    let title, icon, iconLabel;
    if (remaining === 1) {
        title = "...more guess";
        icon = fa1;
        iconLabel = "1";
    } else if (remaining < 10) {
        icon = [fa0, fa1, fa2, fa3, fa4, fa5, fa6, fa7, fa8, fa9][remaining];
        title = "...more guesses";
        iconLabel = `${remaining}`;
    } else {
        // icon has no semantic meaning in this case, so we don't need a label
        icon = faQuestion;
        title = `${remaining} more guesses`;
    }
    return <Card title={title} icon={{ fa: icon, label: iconLabel }} />;
}
