import useSWR from "swr";
import { fetcher } from "../fetcher";
import { Loading, Error } from "./Placeholder";
import { Guesses } from "./Guesses";
import { Nav } from "./Nav";
import { Player } from "./Player";

export function Game() {
    const { data, error, isLoading } = useSWR("/game", fetcher);
    if (error) return <Error error={error} />;
    if (isLoading) return <Loading />;
    // TODO: handle completed games
    return <>
        <Nav />
        <Guesses guesses={data.guesses} />
        <Player unlockedSeconds={data.unlockedSeconds} />
    </>;
}
