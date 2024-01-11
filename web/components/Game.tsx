import useSWR from "swr";
import { fetchGame } from "../api";
import { Loading, Error } from "./Placeholder";
import { Guesses } from "./Guesses";
import { Nav } from "./Nav";
import { Player } from "./Player";
import { GameOver } from "./GameOver";
import { useNavigate } from "react-router-dom";

export function Game() {
    const { data, error } = useSWR("/game", fetchGame);
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
            <Nav />
            {game}
            <Player game={data} />
        </>
    );
}
