import { faInfo, faUser } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Link } from "react-router-dom";

export function Nav() {
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
                    icon={faInfo}
                    title="About and delete"
                    fixedWidth
                />
            </Link>
        </nav>
    );
}
