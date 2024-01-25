import { Link } from "react-router-dom";
import { Game } from "../api";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";
import { TrackCard } from "./TrackCard";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faCrown, faHeartCrack, faPlay } from "@fortawesome/free-solid-svg-icons";

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
    let outcome;
    if (won) {
        const guessPlural = guesses.length === 1 ? "guess" : "guesses";
        const comment = COMMENTS[Math.min(guesses.length, COMMENTS.length) - 1];
        outcome = (
            <div className="card card--header card--good">
                <FontAwesomeIcon className="card__icon" icon={faCrown} />
                <h1 className="card__title">You Won!</h1>
                <div className="card__sub">
                    You took <b>{guesses.length}</b> {guessPlural} - {comment}
                    <br />
                    <GameType game={game} />
                </div>
            </div>
        );
    } else {
        outcome = (
            <div className="card card--header card--bad">
                <FontAwesomeIcon className="card__icon" icon={faHeartCrack} />
                <h1 className="card__title">Game Over</h1>
                <p className="card__sub">
                    <GameType game={game} />
                </p>
            </div>
        );
    }
    return (
        <Scrollable>
            <div className="card_stack">
                {outcome}
                <TrackCard track={track} />
                <Link to="/" className="card card--button">
                    <FontAwesomeIcon className="card__icon" icon={faPlay} />
                    <span className="card__title">New Game</span>
                    <span className="card__sub">Click to play again</span>
                </Link>
            </div>
        </Scrollable>
    );
}
