import { Game } from "../api";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";
import { TrackCard } from "./TrackCard";
import { faCrown, faHeartCrack, faPlay } from "@fortawesome/free-solid-svg-icons";
import { Card } from "./Card";

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
            <Card title={<>You won!&ensp;&bull;&ensp;{type}</>} icon={faCrown} good>
                You took <b>{guesses.length}</b> {guessPlural} - {comment}
            </Card>
        );
    } else {
        outcome = (
            <Card title={<>You Lost&ensp;&bull;&ensp;{type}</>} icon={faHeartCrack} bad>
                But you discovered a new song!
            </Card>
        );
    }
    return (
        <Scrollable>
            <div className="card_stack">
                {outcome}
                <TrackCard track={track} link />
                <Card title="New Game" icon={faPlay} link="/">
                    Click to play again
                </Card>
            </div>
        </Scrollable>
    );
}
