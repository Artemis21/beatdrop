import { useState } from "react";
import useSWR, { useSWRConfig } from "swr";
import { Track, fetchTracks, newGuess } from "../fetcher";

export function SongSearch() {
    const [q, setQ] = useState("");
    const [id, setId] = useState<number | null>(null);
    const [active, setActive] = useState(false);
    const { data, error } = useSWR(
        active ? `/track/search?${query({ q })}` : null,
        fetchTracks,
    );
    let results;
    if (!active || q === "") {
        results = null;
    } else if (error) {
        results = <SearchResultsPlaceholder message={error} />;
    } else if (data === undefined) {
        results = <SearchResultsPlaceholder message="Loading..." />;
    } else if (data.tracks.length == 0) {
        results = <SearchResultsPlaceholder message="No results found." />;
    } else {
        results = (
            <SearchResults tracks={data.tracks} setQ={setQ} setId={setId} />
        );
    }
    let button;
    if (id === null) {
        button = <SkipButton />
    } else {
        button = <GuessButton guess={id} />
    }
    return (
        <div className="guess__title search">
            <div className="search">
                <input
                    className="search__input"
                    type="search"
                    placeholder="Never Gonna..."
                    onChange={e => setQ(e.target.value)}
                    onFocus={() => setActive(true)}
                    onBlur={() => setActive(false)}
                    value={q}
                />
                {results}
            </div>
            {button}
        </div>
    );
}

function query(params: Record<string, any>) {
    const query = new URLSearchParams(params);
    return query.toString();
}

function SearchResults({
    tracks,
    setQ,
    setId,
}: {
    tracks: Track[];
    setQ: (q: string) => void;
    setId: (id: number) => void;
}) {
    return (
        <div className="search__results">
            {tracks.map(track => {
                const displayName = `${track.title} - ${track.artistName}`;
                const click = () => {
                    setId(track.id);
                    setQ(displayName);
                };
                return (
                    <button
                        className="search__results__result"
                        key={track.id}
                        onMouseDown={click}>
                        {displayName}
                    </button>
                );
            })}
        </div>
    );
}

function SearchResultsPlaceholder({ message }: { message: string }) {
    return (
        <div className="search__results">
            <div className="search__results__placeholder">{message}</div>
        </div>
    );
}

function SkipButton() {
    const { mutate } = useSWRConfig();
    const click = async () => {
        await mutate("/game", newGuess(null), { revalidate: false });
    };
    return <button className="guess_button guess_button--skip" onClick={click}>Skip</button>;
}

function GuessButton({ guess }: { guess: number }) {
    const { mutate } = useSWRConfig();
    const click = async () => {
        await mutate("/game", newGuess(guess), { revalidate: false });
    };
    return <button className="guess_button guess_button--guess" onClick={click}>Guess</button>
}
