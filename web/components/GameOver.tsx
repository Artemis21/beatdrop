import { Game } from "../api";
import { GameType } from "./GameType";

export function GameOver({ game }: { game: Game }) {
    const { track, won, guesses } = game;
    if (track === null || won === null) {
        throw new Error("GameOver called on uncompleted game.");
    }
    return (
        <>
            <h1>{won ? "You Won!" : "Game Over :("}</h1>
            <img src={track.albumCover} alt={`Album cover for ${track.albumTitle}`} />
            <p>
                The song was <a href={track.link}>{track.title}</a>
            </p>
            <p>By {track.artistName}</p>
            {won && (
                <p>
                    You took {guesses.length} guess{guesses.length === 1 ? "" : "es"}
                </p>
            )}
            <GameType game={game} />
        </>
    );
}
