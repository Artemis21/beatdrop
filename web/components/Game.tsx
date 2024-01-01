import useSWR from "swr";
import { fetchGame } from "../fetcher";
import { Loading, Error } from "./Placeholder";
import { Guesses } from "./Guesses";
import { Nav } from "./Nav";
import { Player } from "./Player";
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
    // TODO: handle completed games, or game === null
    return (
        <>
            <Nav />
            <div className="guesses">
                <Guesses game={data} />
                <Player game={data} />
            </div>
        </>
    );
}
