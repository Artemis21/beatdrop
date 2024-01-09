import { faChartSimple, faInfo, faUser } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Link } from "react-router-dom";

export function Nav() {
    return (
        <nav className="nav">
            <Link to="/" className="nav__title">
                Beatdrop
            </Link>
            <span className="nav__pad"></span>
            {/* TODO: make these working links */}
            <FontAwesomeIcon className="nav__icon" icon={faChartSimple} fixedWidth />
            <FontAwesomeIcon className="nav__icon" icon={faUser} fixedWidth />
            <FontAwesomeIcon className="nav__icon" icon={faInfo} fixedWidth />
        </nav>
    );
}
