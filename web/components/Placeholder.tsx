// TODO: Improve both of these

export function Loading() {
    return <div className="loading">Loading...</div>;
}

export function Error({ error }: { error: object }) {
    return <div className="error">{error.toString()}</div>;
}
