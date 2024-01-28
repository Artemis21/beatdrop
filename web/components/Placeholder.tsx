import { useRouteError } from "react-router-dom";

// TODO: Improve styling on all of these

export function Loading() {
    return <div className="loading">Loading...</div>;
}

export function Error({ error }: { error: object }) {
    return <div className="error">{error.toString()}</div>;
}

export function ErrorPage({ notFound = false }: { notFound?: boolean }) {
    const error = useRouteError();
    let message;
    if (notFound) {
        message = "Not found";
    } else if (error && typeof error === "string") {
        message = error;
    } else if (error && typeof error === "object") {
        if ("statusText" in error && typeof error.statusText === "string") {
            message = error.statusText;
        } else if ("message" in error && typeof error.message === "string") {
            message = error.message;
        }
    }
    console.error(error);
    return (
        <div>
            <h1 className="title">Uh oh :/</h1>
            <p className="sub">Something&apos;s gone wrong.</p>
            <p className="sub">{message}</p>
        </div>
    );
}
