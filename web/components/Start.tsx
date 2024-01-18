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
import { Scrollable } from "./Scrollable";

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
            <Scrollable>
                <div className="card_stack">{buttons}</div>
            </Scrollable>
        </>
    );
}

function ResumeButton({ id }: { id: number }) {
    return (
        <Link to={`/games/${id}`} className="card card--button">
            <FontAwesomeIcon className="card__thumb" icon={faPlay} fixedWidth />
            <span className="card__title">Resume</span>
            <span className="card__sub">You have an ongoing game</span>
        </Link>
    );
}

function DailyButton({ id }: { id: number | null }) {
    const navigate = useNavigate();
    const { mutate, isLoading } = useNewGame();
    const click = async () => {
        if (id === null) {
            const game = await mutate({ daily: true });
            id = game!.id;
        }
        navigate(`/games/${id}`);
    };
    return (
        <button onClick={isLoading ? undefined : click} className="card card--button">
            <FontAwesomeIcon className="card__thumb" icon={faCalendarDay} fixedWidth />
            <span className="card__title">{isLoading ? "Loading..." : "Daily"}</span>
            <span className="card__sub">
                {id === null ? "Play today's daily game" : "See your results for today"}
            </span>
        </button>
    );
}

function UnlimitedButton() {
    return (
        <Link to="/start/unlimited" className="card card--button">
            <FontAwesomeIcon className="card__thumb" icon={faInfinity} fixedWidth />
            <span className="card__title">Unlimited</span>
            <span className="card__sub">Play as much as you want, or select a genre</span>
        </Link>
    );
}

function TimedButton() {
    return (
        <Link to="/start/timed" className="card card--button">
            <FontAwesomeIcon className="card__thumb" icon={faClock} fixedWidth />
            <span className="card__title">Timed</span>
            <span className="card__sub">
                Submit your guess before the timer runs out!
            </span>
        </Link>
    );
}
