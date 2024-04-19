import { useSearchParams } from "react-router-dom";
import { useNewGame } from "../api";
import { useNavigate } from "react-router-dom";
import { Genre, useGenres } from "../api";
import { Error, Loading } from "./Placeholder";
import { Scrollable } from "./Scrollable";
import { GameType } from "./GameType";
import { RefObject, useRef, useState } from "react";
import { Card } from "./Card";

export function NewGame() {
    const [params] = useSearchParams();
    const timed = params.get("timed") === "true";
    const { data, error } = useGenres();
    const [genre, setGenre] = useState<Genre | null>(null);
    const [filter, setFilter] = useState("");
    const inputEl = useRef<HTMLInputElement>(null);
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
            <label htmlFor="genre_search" className="sub">
                Pick a genre, or leave blank to select randomly
            </label>
            <form className="form_row" onSubmit={e => e.preventDefault()}>
                <Filter
                    filter={filter}
                    setFilter={setFilter}
                    genre={genre}
                    setGenre={setGenre}
                    inputRef={inputEl}
                />
                <StartGame timed={timed} genre={genre} />
            </form>
            <Scrollable>
                <div className="card_stack">
                    {genres.map(g => {
                        return (
                            <Genre
                                key={g.id}
                                genre={g}
                                activeGenre={genre}
                                setActiveGenre={genre => {
                                    setGenre(genre);
                                    inputEl.current?.focus();
                                }}
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
    inputRef,
}: {
    filter: string;
    genre: Genre | null;
    setFilter: (_: string) => void;
    setGenre: (_: Genre | null) => void;
    inputRef: RefObject<HTMLInputElement>;
}) {
    return (
        <input
            className="input"
            type="search"
            placeholder="Alternative Rock..."
            onChange={e => {
                setFilter(e.target.value);
                setGenre(null);
            }}
            value={genre?.name || filter}
            id="genre_search"
            ref={inputRef}
            autoFocus
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
            image={{ src: `${genre.picture}?size=xl`, alt: "" }} // no alt since title is there
            onClick={() => (active ? setActiveGenre(null) : setActiveGenre(genre))}
            active={active}
        />
    );
}
