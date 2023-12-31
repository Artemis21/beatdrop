import { Link } from "react-router-dom";

export function Nav() {
    return <nav className="nav">
        <Link to="/" className="nav__title">Beatdrop</Link>
        <span className="nav__pad"></span>
        {/* TODO: make these working links */}
        <i className="nav__icon fa-solid fa-fw fa-chart-simple"></i>
        <i className="nav__icon fa-solid fa-fw fa-user"></i>
        <i className="nav__icon fa-solid fa-fw fa-info"></i>
    </nav>;
}
