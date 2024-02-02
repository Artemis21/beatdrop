import { IconDefinition } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { MouseEventHandler, ReactNode } from "react";
import { Link } from "react-router-dom";
import { classModifiers } from "../utils";

type ImageDef = {
    src: string;
    alt: string;
};

export function Card({
    title,
    children,
    onClick,
    image,
    icon,
    link,
    good,
    bad,
    active,
    extended,
}: {
    title: ReactNode;
    children?: ReactNode;
    onClick?: MouseEventHandler<HTMLDivElement>;
    image?: ImageDef;
    icon?: IconDefinition;
    link?: string;
    good?: boolean;
    bad?: boolean;
    active?: boolean;
    extended?: boolean;
}) {
    const inner = Inner({ icon, image, title, sub: children });
    const outerClass = classModifiers("card", {
        good,
        bad,
        active,
        extended,
        button: link || onClick,
    });
    if (link && link.startsWith("/")) {
        return (
            <Link to={link} className={outerClass}>
                {inner}
            </Link>
        );
    } else if (link) {
        return (
            <a href={link} className={outerClass}>
                {inner}
            </a>
        );
    } else {
        // FIXME: We should be using a button element if there is onClick, but for
        //        some reason it's causing styling issues on the genre picker.
        return (
            <div className={outerClass} onClick={onClick} role="button">
                {inner}
            </div>
        );
    }
}

function Inner({
    icon,
    image,
    title,
    sub,
}: {
    icon?: IconDefinition;
    image?: ImageDef;
    title?: ReactNode;
    sub?: ReactNode;
}) {
    return (
        <>
            {icon && <FontAwesomeIcon icon={icon} className="card__icon" />}
            {image && (
                <div className="card__image">
                    <img src={image.src} alt={image.alt} />
                </div>
            )}
            <p className="card__title">{title}</p>
            {sub && <div className="card__sub">{sub}</div>}
        </>
    );
}
