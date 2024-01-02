import { Game } from "../api";
import { Icon } from "./Icon";

export function GameType({ game }: { game: Game }) {
    let icon, name, genre;
    if (game.isDaily) {
        icon = "calendar-day";
        name = "Daily";
    } else if (game.isTimed) {
        icon = "clock";
        name = "Timed";
    } else {
        icon = "infinity";
        name = "Unlimited";
    }
    if (game.genre !== null) {
        genre = (
            <>
                <Icon icon="music" /> {game.genre.name}
            </>
        );
    }
    return (
        <>
            <Icon icon={icon} />
            {name}
            {genre}
        </>
    );
}
