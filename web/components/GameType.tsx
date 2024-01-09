import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Game } from "../api";
import {
    faCalendarDay,
    faClock,
    faInfinity,
    faMusic,
} from "@fortawesome/free-solid-svg-icons";

export function GameType({ game: { isDaily, isTimed, genre } }: { game: Game }) {
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
        <>
            <FontAwesomeIcon icon={icon} fixedWidth />
            {name}
            {genre && (
                <>
                    <FontAwesomeIcon icon={faMusic} fixedWidth />
                    {genre.name}
                </>
            )}
        </>
    );
}
