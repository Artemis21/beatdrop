import { useState } from "react";
import useSWR from "swr";
import { Track, fetchTracks } from "../fetcher";

export function SongSearch() {
    const [q, setQ] = useState("");
    const { data, error } = useSWR(
        `/track/search?${query({ q })}`,
        fetchTracks,
    );
    let tracks: Track[];
    // FIXME: display error/loading messages here, without hiding the whole search element.
    if (error) {
        console.error(error);
        tracks = [];
    } else if (data === undefined) {
        tracks = [];
    } else {
        tracks = data.tracks;
    }
    return (
        <>
            <input
                className="guess__title"
                type="search"
                placeholder="Never Gonna..."
                onChange={ e => setQ(e.target.value) }
            />
            {tracks.map(track => (
                <p>{track.title}</p>
            ))}
        </>
    );
}

function query(params: Record<string, any>) {
    const query = new URLSearchParams(params);
    return query.toString();
}
