import { Track } from "../api";

export function TrackCard({ track }: { track: Track }) {
    return <a className="card card--with-image card--button" href={track.link}>
        <div className="card__image">
            <img
                // FIXME: pick size appropriately (small/medium/big/xl)
                src={`${track.albumCover}?size=xl`}
                alt={`Album cover for ${track.albumTitle}`}
            />
        </div>
        <span className="card__title">{track.title}</span>
        <span className="card__sub">{track.artistName}</span>
    </a>
}
