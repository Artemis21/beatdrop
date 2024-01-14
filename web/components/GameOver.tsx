import { Game } from "../api";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";

export function GameOver({ game }: { game: Game }) {
    const { track, won, guesses } = game;
    if (track === null || won === null) {
        throw new Error("GameOver called on uncompleted game.");
    }
    return (
        <Scrollable>
            <div className="game_over game_over--won">
                <h1 className="game_over__title">{won ? "You Won!" : "Game Over :("}</h1>
                <img
                    className="game_over__image"
                    // FIXME: pick size appropriately (small/medium/big/xl)
                    src={`${track.albumCover}?size=xl`}
                    alt={`Album cover for ${track.albumTitle}`}
                />
                <p className="game_over__caption">
                    The song was{" "}
                    <a className="game_over__caption__highlight" href={track.link}>
                        {track.title}
                    </a>
                </p>
                <p className="game_over__caption">By {track.artistName}</p>
                {won && (
                    <p className="game_over__caption">
                        You took {guesses.length} guess{guesses.length === 1 ? "" : "es"}
                    </p>
                )}
                <GameType game={game} />
            </div>
        </Scrollable>
    );
}
