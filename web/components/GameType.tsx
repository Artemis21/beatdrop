import { Game } from "../api";
import { Icon } from "./Icon";

export function GameType({ game }: { game: Game }) {
    if (game.isDaily) {
        return (
            <>
                <Icon icon="calendar-day" />
                Daily
            </>
        );
    }
    let mode;
    if (game.isTimed) {
        mode = (
            <>
                <Icon icon="clock" />
                Timed
            </>
        );
    } else {
        mode = (
            <>
                <Icon icon="infinity" />
                Unlimited
            </>
        );
    }
    if (game.genre !== null) {
        return (
            <>
                {mode} <Icon icon="music" /> {game.genre.name}
            </>
        );
    }
    return mode;
}
