import useSWR, { useSWRConfig } from "swr";
import { fetchGame, newGame } from "../api";
import { useNavigate, Link } from "react-router-dom";
import { Error, Loading } from "./Placeholder";
import { Nav } from "./Nav";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
    faCalendarDay,
    faClock,
    faInfinity,
    faPlay,
} from "@fortawesome/free-solid-svg-icons";

export function Start() {
    const { data, error } = useSWR("/game", fetchGame);
    const { data: daily, error: dailyError } = useSWR("/game/daily", fetchGame);
    if (error || dailyError) return <Error error={error || dailyError} />;
    if (data === undefined || daily === undefined) return <Loading />;
    const buttons = [];
    if (data !== null && data.won === null) {
        buttons.push(<ResumeButton key="resume" />);
        if (daily !== null && daily.won !== null) {
            // Show daily results, since this isn't starting a new game.
            buttons.push(<DailyButton key="daily" />);
        }
    } else {
        buttons.push(<DailyButton key="daily" />);
        buttons.push(<UnlimitedButton key="unlimited" />);
        buttons.push(<TimedButton key="timed" />);
    }
    return (
        <>
            <Nav />
            <div className="stack">{buttons}</div>
        </>
    );
}

function ResumeButton() {
    return (
        <Link to="/game" className="stack__item stack__item--button">
            <FontAwesomeIcon className="stack__item__thumb" icon={faPlay} fixedWidth />
            <span className="stack__item__title">Resume</span>
            <span className="stack__item__sub">You have an ongoing game</span>
        </Link>
    );
}

function DailyButton() {
    const { data, error } = useSWR("/game/daily", fetchGame);
    const navigate = useNavigate();
    const { mutate } = useSWRConfig();
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    // If there is a daily game ongoing, return null since we already have a resume button.
    if (data !== null && data.won === null) return null;
    const click = async () => {
        if (data === null) {
            // `/game` and `/game/daily` refer to the same thing here, so update both caches
            const game = await mutate("/game", newGame({ daily: true }), {
                revalidate: false,
            });
            await mutate("/game/daily", game, { revalidate: false });
        }
        navigate("/game");
    };
    return (
        <button onClick={click} className="stack__item stack__item--button">
            <FontAwesomeIcon
                className="stack__item__thumb"
                icon={faCalendarDay}
                fixedWidth
            />
            <span className="stack__item__title">Daily</span>
            <span className="stack__item__sub">
                {data === null ? "Play today's daily game" : "See your results for today"}
            </span>
        </button>
    );
}

function UnlimitedButton() {
    return (
        <Link to="/start/unlimited" className="stack__item stack__item--button">
            <FontAwesomeIcon
                className="stack__item__thumb"
                icon={faInfinity}
                fixedWidth
            />
            <span className="stack__item__title">Unlimited</span>
            <span className="stack__item__sub">
                Play as much as you want, or select a genre
            </span>
        </Link>
    );
}

function TimedButton() {
    return (
        <Link to="/start/timed" className="stack__item stack__item--button">
            <FontAwesomeIcon className="stack__item__thumb" icon={faClock} fixedWidth />
            <span className="stack__item__title">Timed</span>
            <span className="stack__item__sub">
                Submit your guess before the timer runs out!
            </span>
        </Link>
    );
}
