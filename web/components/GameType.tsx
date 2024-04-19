import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Genre } from "../api";
import {
    faCalendarDay,
    faClock,
    faInfinity,
    faMusic,
} from "@fortawesome/free-solid-svg-icons";

export type GameTypeAttrs = {
    isDaily?: boolean;
    isTimed?: boolean;
    genre?: Genre | null;
};

export function GameType({
    game: { isDaily = false, isTimed = false, genre = null },
    className = "",
}: {
    game: GameTypeAttrs;
    className?: string;
}) {
    let icon, name;
    if (isDaily) {
        icon = faCalendarDay;
        name = "Daily";
    } else if (isTimed) {
        icon = faClock;
        name = "Timed";
    } else {
        icon = faInfinity;
        name = "Unlimited";
    }
    return (
        <span className={`game_type ${className}`}>
            <span className="game_type">
                <FontAwesomeIcon icon={icon} className="game_type__icon" />
                {name}
            </span>
            {genre && (
                    <span className="game_type">
                    <FontAwesomeIcon icon={faMusic} className="game_type__icon" />
                    {genre.name}
                    </span>
            )}
        </span>
    );
}
