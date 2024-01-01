import useSWR from "swr";
import { fetchBlob, Game } from "../api";
import { Error, Loading } from "./Placeholder";
import { useState } from "react";

export function Player({ game }: { game: Game }) {
    const { data, error } = useSWR("/game/clip", fetchBlob);
    const [currentTime, setCurrentTime] = useState(0);
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    const url = URL.createObjectURL(data);
    const audio = new Audio(url);
    audio.ontimeupdate = () => setCurrentTime(audio.currentTime * 1000);
    return (
        <>
            <TrackBar currentTime={currentTime} game={game} />
            <Controls currentTime={currentTime} audio={audio} game={game} />
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
    audio,
    game,
}: {
    currentTime: number;
    audio: HTMLMediaElement;
    game: Game;
}) {
    const seekPoints = [
        0,
        ...game.constants.musicClipMillis.slice(0, game.guesses.length + 1),
    ];
    // FIXME: these forward/backward buttons don't seem to work very well
    return (
        <div className="controls">
            <BackButton currentTime={currentTime} audio={audio} seekPoints={seekPoints} />
            <PlayButton audio={audio} />
            <ForwardButton
                currentTime={currentTime}
                audio={audio}
                seekPoints={seekPoints}
            />
        </div>
    );
}

function BackButton({
    currentTime,
    audio,
    seekPoints,
}: {
    currentTime: number;
    audio: HTMLMediaElement;
    seekPoints: number[];
}) {
    const icon = <i className="fa-solid fa-fw fa-rotate-left"></i>;
    const seekTo = seekPoints.filter(time => time < currentTime).pop();
    if (seekTo === undefined) {
        return <div className="control control--disabled">{icon}</div>;
    }
    return (
        <div
            className="control control--enabled"
            onClick={() => audio.fastSeek(seekTo * 1000)}
        >
            {icon}
        </div>
    );
}

function ForwardButton({
    currentTime,
    audio,
    seekPoints,
}: {
    currentTime: number;
    audio: HTMLMediaElement;
    seekPoints: number[];
}) {
    const icon = <i className="fa-solid fa-fw fa-rotate-right"></i>;
    const seekTo = seekPoints.filter(time => time > currentTime).shift();
    if (seekTo === undefined) {
        return <div className="control control--disabled">{icon}</div>;
    }
    return (
        <div
            className="control control--enabled"
            onClick={() => audio.fastSeek(seekTo * 1000)}
        >
            {icon}
        </div>
    );
}

function PlayButton({ audio }: { audio: HTMLMediaElement }) {
    let icon, click;
    if (audio.paused) {
        icon = <i className="fa-solid fa-fw fa-play"></i>;
        click = () => audio.play();
    } else {
        icon = <i className="fa-solid fa-fw fa-play"></i>;
        click = () => audio.pause;
    }
    return (
        <div className="control control--enabled control--play" onClick={click}>
            {icon}
        </div>
    );
}
