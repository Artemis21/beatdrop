import { useNewGame, useRecentGames } from "../api";
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
    const { data, error } = useRecentGames();
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    const buttons = [];
    if (data.ongoing !== null) {
        buttons.push(<ResumeButton key="resume" id={data.ongoing} />);
        if (data.daily !== null && data.daily !== data.ongoing) {
            // we can't show any "start game" buttons with an ongoing game, but
            // we can show a button to view daily results if the daily game was
            // already completed
            buttons.push(<DailyButton key="daily" id={data.daily} />);
        }
    } else {
        buttons.push(<DailyButton key="daily" id={data.daily} />);
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

function ResumeButton({ id }: { id: number }) {
    return (
        <Link to={`/games/${id}`} className="stack__item stack__item--button">
            <FontAwesomeIcon className="stack__item__thumb" icon={faPlay} fixedWidth />
            <span className="stack__item__title">Resume</span>
            <span className="stack__item__sub">You have an ongoing game</span>
        </Link>
    );
}

function DailyButton({ id }: { id: number | null }) {
    const navigate = useNavigate();
    const newGame = useNewGame();
    const click = async () => {
        if (id === null) {
            const game = await newGame({ daily: true });
            id = game!.id;
        }
        navigate(`/games/${id}`);
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
                {id === null ? "Play today's daily game" : "See your results for today"}
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
