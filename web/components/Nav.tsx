import { Link } from "react-router-dom";
import { Icon } from "./Icon";

export function Nav() {
    return (
        <nav className="nav">
            <Link to="/" className="nav__title">
                Beatdrop
            </Link>
            <span className="nav__pad"></span>
            {/* TODO: make these working links */}
            <Icon className="nav__icon" icon="chart-simple" />
            <Icon className="nav__icon" icon="user" />
            <Icon className="nav__icon" icon="info" />
        </nav>
    );
}
