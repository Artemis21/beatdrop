import useSWR from "swr";
import { fetchAudio, Game } from "../api";
import { Error, Loading } from "./Placeholder";
import { useEffect, useState } from "react";

export function Player({ game }: { game: Game }) {
    const { data: audio, error } = useSWR("/game/clip", fetchAudio);
    const [seek, setSeek] = useState(0);
    const [paused, setPaused] = useState(true);
    const [currentTime, setCurrentTime] = useState(0);
    useEffect(() => {
        if (error || audio === undefined) return;
        audio.currentTime = seek / 1000;
        if (!paused) audio.play();
        const handleTimeUpdate = () => setCurrentTime(audio.currentTime * 1000);
        audio.addEventListener("timeupdate", handleTimeUpdate);
        return () => {
            audio.removeEventListener("timeupdate", handleTimeUpdate);
            audio.pause();
        };
    }, [seek, paused, audio]);
    if (error) return <Error error={error} />;
    if (audio === undefined) return <Loading />;
    return (
        <>
            <TrackBar currentTime={currentTime} game={game} />
            <Controls
                currentTime={currentTime}
                setSeek={setSeek}
                paused={paused}
                setPaused={setPaused}
                game={game}
            />
        </>
    );
}

function TrackBar({ currentTime, game }: { currentTime: number; game: Game }) {
    const { guesses, constants } = game;
    const segments = [];
    let columnWidths = "";
    let lastClipLength = 0;
    const unlockedSegments = guesses.length + 1;
    for (let n = 0; n < constants.maxGuesses; n++) {
        const clipLength = constants.musicClipMillis[n];
        const segmentWidth = clipLength - lastClipLength;
        columnWidths += `${segmentWidth}fr `;
        if (unlockedSegments > n) {
            let timeIntoSegment = currentTime - lastClipLength;
            if (timeIntoSegment < 0) timeIntoSegment = 0;
            if (timeIntoSegment > segmentWidth) timeIntoSegment = segmentWidth;
            const progressWidth = (100 * timeIntoSegment) / segmentWidth;
            segments.push(<UnlockedSegment key={n} progressWidth={progressWidth} />);
        } else {
            segments.push(<LockedSegment key={n} />);
        }
        lastClipLength = clipLength;
    }
    return (
        <div className="play_bar" style={{ gridTemplateColumns: columnWidths }}>
            {segments}
        </div>
    );
}

function UnlockedSegment({ progressWidth }: { progressWidth: number }) {
    return (
        <div className="play_bar__seg play_bar__seg--unlocked">
            <div className="play_bar__seg__fill" style={{ width: `${progressWidth}%` }} />
        </div>
    );
}

function LockedSegment() {
    return <div className="play_bar__seg play_bar__seg--locked"></div>;
}

function Controls({
    currentTime,
    setSeek,
    paused,
    setPaused,
    game,
}: {
    currentTime: number;
    setSeek: (_: number) => void;
    paused: boolean;
    setPaused: (_: boolean) => void;
    game: Game;
}) {
    const seekPoints = [
        0,
        ...game.constants.musicClipMillis.slice(0, game.guesses.length + 1),
    ];
    return (
        <div className="controls">
            <BackButton
                currentTime={currentTime}
                setSeek={setSeek}
                seekPoints={seekPoints}
            />
            <PlayButton
                currentTime={currentTime}
                setSeek={setSeek}
                paused={paused}
                setPaused={setPaused}
            />
            <ForwardButton
                currentTime={currentTime}
                setSeek={setSeek}
                seekPoints={seekPoints}
            />
        </div>
    );
}

function BackButton({
    currentTime,
    setSeek,
    seekPoints,
}: {
    currentTime: number;
    setSeek: (_: number) => void;
    seekPoints: number[];
}) {
    const icon = <i className="fa-solid fa-fw fa-rotate-left"></i>;
    const seekTo = seekPoints.filter(time => time < currentTime).pop();
    if (seekTo === undefined) {
        return <div className="control control--disabled">{icon}</div>;
    }
    return (
        <div className="control control--enabled" onClick={() => setSeek(seekTo)}>
            {icon}
        </div>
    );
}

function ForwardButton({
    currentTime,
    setSeek,
    seekPoints,
}: {
    currentTime: number;
    setSeek: (_: number) => void;
    seekPoints: number[];
}) {
    const icon = <i className="fa-solid fa-fw fa-rotate-right"></i>;
    const seekTo = seekPoints.filter(time => time > currentTime).shift();
    if (seekTo === undefined) {
        return <div className="control control--disabled">{icon}</div>;
    }
    return (
        <div className="control control--enabled" onClick={() => setSeek(seekTo)}>
            {icon}
        </div>
    );
}

function PlayButton({
    currentTime,
    paused,
    setPaused,
    setSeek,
}: {
    currentTime: number;
    paused: boolean;
    setPaused: (_: boolean) => void;
    setSeek: (_: number) => void;
}) {
    let icon, click;
    if (paused) {
        icon = <i className="fa-solid fa-fw fa-play"></i>;
        click = () => setPaused(false);
    } else {
        icon = <i className="fa-solid fa-fw fa-pause"></i>;
        click = () => {
            setPaused(true);
            setSeek(currentTime);
        };
    }
    return (
        <div className="control control--enabled control--play" onClick={click}>
            {icon}
        </div>
    );
}
