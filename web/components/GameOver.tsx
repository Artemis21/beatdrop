import { Game } from "../api";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";
import { TrackCard } from "./TrackCard";
import { faCrown, faHeartCrack, faPlay } from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";
import { Attribution } from "./Attribution";

const COMMENTS = [
    "wow!!",
    "unbelievable!",
    "nice one.",
    "pretty good.",
    "not bad.",
    "eh, ok.",
    "at least you got there in the end...",
];

export function GameOver({ game }: { game: Game }) {
    const { track, won, guesses } = game;
    if (track === null || won === null) {
        throw new Error("GameOver called on uncompleted game.");
    }
    const type = <GameType game={game} className="card__title__tag" />;
    let outcome;
    if (won) {
        const guessPlural = guesses.length === 1 ? "guess" : "guesses";
        const comment = COMMENTS[Math.min(guesses.length, COMMENTS.length) - 1];
        outcome = (
            <Card
                icon={faCrown}
                title={<>You won! &ndash; {type}</>}
                // prettier-ignore
                details={<>You took <b>{guesses.length} {guessPlural}</b> - {comment}</>}
                good
            />
        );
    } else {
        outcome = (
            <Card
                icon={faHeartCrack}
                title={<>You Lost &ndash; {type}</>}
                details="But you discovered a new song!"
                bad
            />
        );
    }
    return (
        <Scrollable>
            <div className="card_stack">
                {outcome}
                <TrackCard track={track} link />
                <Attribution />
                <Card
                    icon={faPlay}
                    title="New Game"
                    details="Click to play again"
                    link="/"
                />
            </div>
        </Scrollable>
    );
}
