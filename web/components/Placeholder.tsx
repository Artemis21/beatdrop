// TODO: Improve both of these

export function Loading() {
    return <div className="loading">Loading...</div>;
}

export function Error({ error }) {
    return <div className="error">{ error.toString() }</div>;
}
