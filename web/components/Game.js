import useSWR from "swr";
import { Loading, Error } from "./Placeholder";
import { Guesses } from "./Guesses";

export function Game() {
    const { data, error, isLoading } = useSWR("/game");
    if (error) return <Error error={error} />;
    if (isLoading) return <Loading />;
    return <>
        <Guesses guesses={data} />
        <PlayBar />
        <Controls />
    </>;
}
