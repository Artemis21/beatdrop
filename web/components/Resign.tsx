import { useResignGame } from "../api";
import { classModifiers } from "../utils";

export function Resign({ gameId }: { gameId: number }) {
    const { mutate, isLoading } = useResignGame();
    const className = classModifiers("submit", { danger: true });
    if (isLoading) return <button className={className}>...</button>;
    const onClick = async () => {
        if (confirm("Are you sure you want to resign this game?")) {
            await mutate({ gameId });
        }
    };
    return (
        <button className={className} onClick={onClick}>
            Give Up
        </button>
    );
}
