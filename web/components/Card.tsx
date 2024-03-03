import { IconDefinition } from "@fortawesome/free-solid-svg-icons";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { MouseEventHandler, ReactNode } from "react";
import { Link } from "react-router-dom";
import { classModifiers } from "../utils";

type ImageDef = {
    src: string;
    alt: string;
};

type IconProps = {
    fa: IconDefinition;
    label?: string;
};

type IconDef = IconProps | IconDefinition;

type Flags = {
    good?: boolean;
    bad?: boolean;
    active?: boolean;
    extended?: boolean;
    centred?: boolean;
};

type CardProps = {
    title?: ReactNode;
    details?: ReactNode;
    children?: ReactNode;
    onClick?: MouseEventHandler<HTMLButtonElement>;
    image?: ImageDef;
    icon?: IconDef;
    link?: string;
    progress?: number;
} & Flags;

export function Card({
    title,
    details,
    children,
    onClick,
    image,
    icon,
    link,
    progress,
    ...flags
}: CardProps) {
    const inner = <Inner {...{ icon, image, title, details, progress, children }} />;
    const outerClass = classModifiers("card", { button: link || onClick, ...flags });
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
    } else if (onClick) {
        return (
            <button className={outerClass} onClick={onClick}>
                {inner}
            </button>
        );
    } else {
        return <div className={outerClass}>{inner}</div>;
    }
}

function Inner({
    icon,
    image,
    title,
    details,
    progress,
    children,
}: {
    icon?: IconDef;
    image?: ImageDef;
    title?: ReactNode;
    details?: ReactNode;
    progress?: number;
    children?: ReactNode;
}) {
    if (icon && !("fa" in icon)) {
        icon = { fa: icon };
    }
    return (
        <>
            {progress !== undefined && (
                <div
                    className="card__progress"
                    style={{ width: `${Math.max(0, Math.min(1, progress)) * 100}%` }}
                />
            )}
            {icon && (
                <div className="card__icon">
                    <FontAwesomeIcon icon={icon.fa} title={icon.label} />
                </div>
            )}
            {image && (
                <div className="card__image">
                    <img src={image.src} alt={image.alt} />
                </div>
            )}
            <div className="card__body">
                {title && <h2 className="card__title">{title}</h2>}
                {details && <p className="card__details">{details}</p>}
                {children}
            </div>
        </>
    );
}
