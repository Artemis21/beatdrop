import { useGame } from "../api";
import { Loading, Error } from "./Placeholder";
import { Guesses } from "./Guesses";
import { Player } from "./Player";
import { GameOver } from "./GameOver";
import { useNavigate, useParams } from "react-router-dom";

export function Game() {
    // TODO: implement timed games
    const { gameId } = useParams();
    const { data, error } = useGame(Number(gameId));
    const navigate = useNavigate();
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    if (data === null) {
        navigate("/");
        return <Loading />;
    }
    let game;
    if (data.won === null) {
        game = <Guesses game={data} />;
    } else {
        game = <GameOver game={data} />;
    }
    return (
        <>
            {game}
            <Player game={data} />
        </>
    );
}
