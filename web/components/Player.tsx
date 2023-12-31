import useSWR from "swr";
import { fetchBlob, Game } from "../fetcher";
import { Error, Loading } from "./Placeholder";
import { useState } from "react";

export function Player({ game }: { game: Game }) {
    const { data, error } = useSWR("/game/clip", fetchBlob);
    const [ currentTime, setCurrentTime ] = useState(0);
    if (error) return <Error error={error} />;
    if (data === undefined) return <Loading />;
    const url = URL.createObjectURL(data);
    const audio = new Audio(url);
    audio.ontimeupdate = () => setCurrentTime(audio.currentTime * 1000);
    return <>
        <button onClick={() => audio.play()}>Play</button>
        <TrackBar currentTime={currentTime} game={game} />
    </>;
}

function TrackBar({ currentTime, game }: { currentTime: number, game: Game }) {
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
            const progressWidth = 100 * timeIntoSegment / segmentWidth;
            segments.push(<UnlockedSegment key={n} progressWidth={progressWidth} />)
        } else {
            segments.push(<LockedSegment key={n} />)
        }
        lastClipLength = clipLength;
    }
    return <div className="play_bar" style={{ gridTemplateColumns: columnWidths }}>
        { segments }
    </div>;
}

function UnlockedSegment({ progressWidth }: { progressWidth: number }) {
    return <div className="play_bar__seg play_bar__seg--unlocked">
        <div className="play_bar__seg__fill" style={{ width: `${progressWidth}%` }} />
    </div>;
}

function LockedSegment() {
    return <div className="play_bar__seg play_bar__seg--locked"></div>;
}
