import { StrictMode } from "react";
import {
    createBrowserRouter,
    createRoutesFromElements,
    Outlet,
    Route,
    RouterProvider,
} from "react-router-dom";
import { Start } from "./Start";
import { Game } from "./Game";
import { NewGame } from "./NewGame";
import { ErrorPage } from "./Placeholder";
import { Nav } from "./Nav";

export function App() {
    const router = createBrowserRouter(
        createRoutesFromElements(
            <Route path="/" element={<Root />}>
                <Route errorElement={<ErrorPage />}>
                    <Route index element={<Start />} />
                    <Route path="/start" element={<NewGame />} />
                    <Route path="/games/:gameId" element={<Game />} />
                    <Route path="*" element={<ErrorPage notFound />} />
                </Route>
            </Route>,
        ),
    );
    return (
        <StrictMode>
            <RouterProvider router={router} />
        </StrictMode>
    );
}

export function Root() {
    return (
        <>
            <Nav />
            <Outlet />
        </>
    );
}
