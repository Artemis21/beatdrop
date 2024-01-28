import { useSearchParams } from "react-router-dom";
import { Genre, useGenres } from "../api";
import { Error, Loading } from "./Placeholder";
import { Nav } from "./Nav";
import { Scrollable } from "./Scrollable";
import { GameType } from "./GameType";
import { useState } from "react";
import { Card } from "./Card";

export function NewGame() {
    // TODO: refactor
    const [params] = useSearchParams();
    const timed = params.get("timed") === "true";
    const { data, error } = useGenres();
    const [genre, setGenre] = useState<Genre | null>(null);
    const [filter, setFilter] = useState("");
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    const genres = data.genres.filter(
        g => g.id === genre?.id || g.name.toLowerCase().includes(filter.toLowerCase()),
    );
    return (
        <>
            <Nav />
            <h1 className="title">New Game</h1>
            <h2 className="sub">
                <GameType game={{ isDaily: false, isTimed: timed, genre }} />
            </h2>
            <label htmlFor="genre_search">
                Pick a genre, or leave blank to select randomly
            </label>
            <div className="search_and_submit">
                <input
                    className="search__input"
                    type="search"
                    placeholder="Alternative Rock..."
                    onChange={e => setFilter(e.target.value)}
                    value={filter}
                    id="genre_search"
                />
                <button className="submit">Start</button>
            </div>
            <Scrollable>
                <div className="card_stack">
                    {genres.map(g => {
                        return (
                            <Card
                                key={g.id}
                                title={g.name}
                                image={{ src: `${g.picture}?size=xl`, alt: g.name }}
                                onClick={() =>
                                    g.id === genre?.id ? setGenre(null) : setGenre(g)
                                }
                                active={g.id === genre?.id}
                            />
                        );
                    })}
                </div>
            </Scrollable>
        </>
    );
}
