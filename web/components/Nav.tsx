import { faInfo, faUser } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { Link } from "react-router-dom";

export function Nav() {
    return (
        <nav className="nav">
            <Link to="/" className="nav__title">
                Beatdrop
            </Link>
            <span className="nav__pad"></span>
            {/* TODO: actually add stats
                <FontAwesomeIcon className="nav__icon" icon={faChartSimple} fixedWidth />
            */}
            <Link to="/user" className="nav__icon">
                <FontAwesomeIcon className="nav__icon" icon={faUser} fixedWidth />
            </Link>
            <Link to="/info" className="nav__icon">
                <FontAwesomeIcon className="nav__icon" icon={faInfo} fixedWidth />
            </Link>
        </nav>
    );
}
