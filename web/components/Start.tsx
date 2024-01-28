import { useNewGame, useRecentGames } from "../api";
import { useNavigate } from "react-router-dom";
import { Error, Loading } from "./Placeholder";
import {
    faCalendarDay,
    faClock,
    faInfinity,
    faPlay,
} from "@fortawesome/free-solid-svg-icons";
import { Scrollable } from "./Scrollable";
import { Card } from "./Card";

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
        <Scrollable>
            <div className="card_stack">{buttons}</div>
        </Scrollable>
    );
}

function ResumeButton({ id }: { id: number }) {
    return (
        <Card title="Resume" icon={faPlay} link={`/games/${id}`}>
            You have an ongoing game
        </Card>
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
    const sub = id === null ? "Play today's daily game" : "See your results for today";
    if (isLoading) {
        return (
            <Card title="Loading..." icon={faCalendarDay}>
                {sub}
            </Card>
        );
    } else {
        return (
            <Card title="Daily" icon={faCalendarDay} onClick={click}>
                {sub}
            </Card>
        );
    }
}

function UnlimitedButton() {
    return (
        <Card title="Unlimited" icon={faInfinity} link="/start?timed=false">
            Play as much as you want or picka a genre
        </Card>
    );
}

function TimedButton() {
    return (
        <Card title="Timed" icon={faClock} link="/start?timed=true">
            Make your guess before the timer runs out!
        </Card>
    );
}
