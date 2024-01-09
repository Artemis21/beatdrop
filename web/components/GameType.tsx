import { Game } from "../api";
import { Icon } from "./Icon";

export function GameType({ game: { isDaily, isTimed, genre } }: { game: Game }) {
    let icon, name;
    if (isDaily) {
        icon = "calendar-day";
        name = "Daily";
    } else if (isTimed) {
        icon = "clock";
        name = "Timed";
    } else {
        icon = "infinity";
        name = "Unlimited";
    }
    return (
        <>
            <Icon icon={icon} />
            {name}
            {genre && (
                <>
                    <Icon icon="music" />
                    {genre.name}
                </>
            )}
        </>
    );
}
