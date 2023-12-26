import { StrictMode } from "react";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Nav } from "./Nav";
import { Start } from "./Start";
import { Game } from "./Game";

export function App() {
    const router = createBrowserRouter([
        {
            path: "/",
            element: <Start />
        },
        {
            path: "/game",
            element: <Game />
        },
    ]);
    return <StrictMode>
        <Nav />
        <RouterProvider router={router} />
    </StrictMode>;
}
