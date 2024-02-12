import { Card } from "./Card";
import wordMark from "../deezer_wordmark.png";

export function Attribution() {
    // If https://deezerbrand.com ever gets updated, this should be changed to follow those guidelines.
    return (
        <Card title="Music from" link="https://deezer.com" centred>
            <img className="deezer_wordmark" src={wordMark} alt="Deezer" />
        </Card>
    );
}
