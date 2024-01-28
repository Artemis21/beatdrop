import { Track } from "../api";
import { Card } from "./Card";

export function TrackCard({ track }: { track: Track }) {
    const image = {
        // FIXME: pick size appropriately (small/medium/big/xl)
        src: `${track.albumCover}?size=xl`,
        alt: `Album cover for ${track.albumTitle}`,
    };
    return (
        <Card title={track.title} image={image} link={track.link}>
            {track.artistName}
        </Card>
    );
}
