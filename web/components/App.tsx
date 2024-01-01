import { StrictMode } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Start } from "./Start";
import { Game } from "./Game";

export function App() {
    const router = createBrowserRouter([
        {
            path: "/",
            element: <Start />,
        },
        {
            path: "/game",
            element: <Game />,
        },
        // TODO: `/start/timed` and `/start/unlimited` routes
        // TODO: handle 404s
    ]);
    return (
        <StrictMode>
            <RouterProvider router={router} />
        </StrictMode>
    );
}
