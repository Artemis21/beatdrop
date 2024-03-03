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
        alt: "", // just decorative
    };
    const linkProp = link ? track.link : undefined;
    return (
        <Card
            image={image}
            title={track.title}
            details={track.artistName}
            link={linkProp}
            bad={bad}
        />
    );
}
