import { Track } from "../api";
import { Card } from "./Card";

export function TrackCard({
    track,
    link = false,
    bad = false,
}: {
    track: Track;
    link?: boolean;
    bad?: boolean;
}) {
    const image = {
        // FIXME: pick size appropriately (small/medium/big/xl)
        src: `${track.albumCover}?size=xl`,
        alt: `Album cover for ${track.albumTitle}`,
    };
    const linkProp = link ? track.link : undefined;
    return (
        <Card title={track.title} image={image} link={linkProp} bad={bad}>
            {track.artistName}
        </Card>
    );
}
