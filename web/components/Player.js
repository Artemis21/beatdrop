import useSWR from "swr";
import { fetcher } from "../fetcher";
import { Error, Loading } from "./Placeholder";

export function Player({ unlockedSeconds }) {
    const { data, error, isLoading } = useSWR("/game/clip", fetcher);
    if (error) return <Error error={error} />;
    if (isLoading) return <Loading />;
    console.log(data);
    const url = URL.createObjectURL(data);
    const audio = new Audio(url);
    return <button onClick={() => audio.play()}>Play</button>;
}
