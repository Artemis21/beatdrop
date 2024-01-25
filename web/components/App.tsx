import { StrictMode } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Start } from "./Start";
import { Game } from "./Game";
import { NewGame } from "./NewGame";

export function App() {
    const router = createBrowserRouter([
        {
            path: "/",
            element: <Start />,
        },
        {
            path: "/games/:gameId",
            element: <Game />,
        },
        {
            path: "/start",
            element: <NewGame />,
        },
        // TODO: `/start/timed` and `/start/unlimited` routes
        // TODO: handle 404s
        // TODO: error handling
    ]);
    return (
        <StrictMode>
            <RouterProvider router={router} />
        </StrictMode>
    );
}
