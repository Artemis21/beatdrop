import { useSearchParams } from "react-router-dom";
import { useNewGame } from "../api";
import { useNavigate } from "react-router-dom";
import { Genre, useGenres } from "../api";
import { Error, Loading } from "./Placeholder";
import { Scrollable } from "./Scrollable";
import { GameType } from "./GameType";
import { useState } from "react";
import { Card } from "./Card";

export function NewGame() {
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
            <h1 className="title">New Game</h1>
            <h2 className="sub">
                <GameType game={{ isDaily: false, isTimed: timed, genre }} />
            </h2>
            <label htmlFor="genre_search">
                Pick a genre, or leave blank to select randomly
            </label>
            <div className="search_and_submit">
                <Filter
                    filter={filter}
                    setFilter={setFilter}
                    genre={genre}
                    setGenre={setGenre}
                />
                <StartGame timed={timed} genre={genre} />
            </div>
            <Scrollable>
                <div className="card_stack">
                    {genres.map(g => {
                        return (
                            <Genre
                                key={g.id}
                                genre={g}
                                activeGenre={genre}
                                setActiveGenre={setGenre}
                            />
                        );
                    })}
                </div>
            </Scrollable>
        </>
    );
}

export function StartGame({ timed, genre }: { timed: boolean; genre: Genre | null }) {
    const navigate = useNavigate();
    const { mutate, isLoading } = useNewGame();
    if (isLoading) {
        return <button className="submit">...</button>;
    }
    const startGame = async () => {
        const game = await mutate({ timed, genreId: genre?.id });
        navigate(`/games/${game!.id}`);
    };
    return (
        <button className="submit" onClick={startGame}>
            Start
        </button>
    );
}

export function Filter({
    filter,
    genre,
    setFilter,
    setGenre,
}: {
    filter: string;
    genre: Genre | null;
    setFilter: (_: string) => void;
    setGenre: (_: Genre | null) => void;
}) {
    return (
        <input
            className="search__input"
            type="search"
            placeholder="Alternative Rock..."
            onChange={e => {
                setFilter(e.target.value);
                setGenre(null);
            }}
            value={genre?.name || filter}
            id="genre_search"
        />
    );
}

export function Genre({
    activeGenre,
    setActiveGenre,
    genre,
}: {
    activeGenre: Genre | null;
    setActiveGenre: (_: Genre | null) => void;
    genre: Genre;
}) {
    const active = genre.id === activeGenre?.id;
    return (
        <Card
            title={genre.name}
            image={{ src: `${genre.picture}?size=xl`, alt: genre.name }}
            onClick={() => (active ? setActiveGenre(null) : setActiveGenre(genre))}
            active={active}
        />
    );
}
