import { useNavigate } from "react-router-dom";
import { GameType } from "./GameType";
import { Scrollable } from "./Scrollable";
import { Card } from "./Card";
import { faCrown, faForward, faLock, faMusic, faPlay, faRotateRight, faSearch } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";

export function Info() {
    const navigate = useNavigate();
    const daily = <GameType game={{ isDaily: true }} />;
    return (
        <Scrollable>
            <div className="card_stack card_stack--centred">
                <Card title="How to Play" extended>
                    <ol className="fa-ul">
                        <li>
                            <FontAwesomeIcon icon={faPlay} listItem />
                            Start a game - try <b>{daily}</b> for your first time.
                        </li>
                        <li>
                            <FontAwesomeIcon icon={faMusic} listItem />
                            Listen to the music. Does it ring a bell?</li>
                        <li>
                            <FontAwesomeIcon icon={faSearch} listItem />
                            Start typing your guess, then pick a search result.</li>
                        <li>
                            <FontAwesomeIcon icon={faLock} listItem />
                            Incorrect guesses let you hear more of the track.</li>
                        <li>
                            <FontAwesomeIcon icon={faForward} listItem />
                            If you have no idea, you can skip your guess.</li>
                        <li>
                            <FontAwesomeIcon icon={faRotateRight} listItem />
                            Use the buttons at the bottom of the page to jump backwards
                            and forwards.
                        </li>
                        <li>
                            <FontAwesomeIcon icon={faCrown} listItem />
                            Try to get the song before your guesses run out! Good luck!
                        </li>
                    </ol>
                </Card>
                <Card title="About" extended>
                    <p>
                        <b>Beatdrop</b> is a name-that-tune music guessing game inspired
                        by the now-discontinued Heardle.
                    </p>
                    <p>
                        Music metadata, album art and track audio are provided by{" "}
                        <a href="https://deezer.com">Deezer</a>.
                    </p>
                    <p>
                        Beatdrop is a personal project by AM Vellacott, and is not
                        affiliated with Deezer, Spotify, Heardle, or any other music
                        service.{" "}
                    </p>
                    <p>
                        <a href="https://github.com/Artemis21/beatdrop">
                            The source code is available on GitHub.
                        </a>
                    </p>
                </Card>
                <Card title="Start playing!" icon={faPlay} onClick={() => navigate(-1)} />
            </div>
        </Scrollable>
    );
}
