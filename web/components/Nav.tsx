import { faUser, faQuestion } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Link } from "react-router-dom";
import { useState } from "react";

export function Nav() {
    const [seenInfo, setSeenInfo] = useState(() => {
        return localStorage.getItem("seenInfo") === "true";
    });
    return (
        <nav className="nav">
            <Link to="/" className="nav__title nav__button">
                Beatdrop
            </Link>
            <span className="nav__pad"></span>
            {/* TODO: actually add stats
                <FontAwesomeIcon className="nav__icon" icon={faChartSimple} fixedWidth />
            */}
            <Link to="/user" className="nav__button">
                <FontAwesomeIcon
                    className="nav__icon"
                    icon={faUser}
                    title="Manage account"
                    fixedWidth
                />
            </Link>
            <Link to="/info" className="nav__button">
                <FontAwesomeIcon
                    className="nav__icon"
                    icon={faQuestion}
                    title={seenInfo ? "About" : "Unread: About"}
                    onClick={() => {
                        setSeenInfo(true);
                        localStorage.setItem("seenInfo", "true");
                    }}
                    fixedWidth
                />
                {!seenInfo && <div className="nav__button__badge"></div>}
            </Link>
        </nav>
    );
}
