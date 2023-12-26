import useSWR from "swr";
import { fetcher, newGame } from "../fetcher";
import { useNavigate, Link } from "react-router-dom";
import { Error, Loading } from "./Placeholder";
import { Nav } from "./Nav";

export function Start() {
    const { data: game, isLoading: gameIsLoading, error: gameError } = useSWR("/game", fetcher);
    const { data: dailyGame, isLoading: dailyIsLoading, error: dailyError } = useSWR("/game/daily", fetcher);
    if (gameError) return <Error error={gameError} />;
    if (dailyError) return <Error error={dailyError} />;
    if (gameIsLoading) return <Loading />;
    if (dailyIsLoading) return <Loading />;
    const buttons = [];
    if (game !== null && game.won === null) {
        buttons.push(<ResumeButton key="resume" />);
    } else {
        if (dailyGame === null) {
            buttons.push(<DailyButton key="daily" />);
        }
        buttons.push(<UnlimitedButton key="unlimited" />);
        buttons.push(<TimedButton key="timed" />);
    }
    return <>
        <Nav />
        <div className="start_buttons">{buttons}</div>
    </>;
}

function ResumeButton() {
    return <Link to="/game" className="start_button">
        <i className="start_button__icon fa-solid fa-fw fa-play"></i>
        <span className="start_button__title">Resume</span>
        <span className="start_button__sub">You have an ongoing game</span>
    </Link>;
}

function DailyButton() {
    const navigate = useNavigate();
    const click = () => newGame({ daily: true }).then(() => navigate("/game"));
    return <button onClick={click} className="start_button">
        <i className="start_button__icon fa-solid fa-fw fa-calendar-day"></i>
        <span className="start_button__title">Daily</span>
        <span className="start_button__sub">A new game every day</span>
    </button>;
}

function UnlimitedButton() {
    return <Link to="/start/unlimited" className="start_button">
        <i className="start_button__icon fa-solid fa-fw fa-infinity"></i>
        <span className="start_button__title">Unlimited</span>
        <span className="start_button__sub">Play as much as you want, or select a genre</span>
    </Link>;
}

function TimedButton() {
    return <Link to="/start/timed" className="start_button">
        <i className="start_button__icon fa-solid fa-fw fa-clock"></i>
        <span className="start_button__title">Timed</span>
        <span className="start_button__sub">Submit your guess before the timer runs out!</span>
    </Link>;
}
