import { useState } from "react";
import useSWR, { Key, MutatorOptions, useSWRConfig } from "swr";

const API_URL = "/api";

/** Singleton class which handles storing credentials in local storage. */
class Credentials {
    /** The login secret, used to authenticate an anonymous account to create sessions. */
    #loginSecret = localStorage.getItem("loginSecret");

    /** The session token, used to authenticate requests. */
    #sessionToken = localStorage.getItem("sessionToken");

    constructor() {
        this.#loginSecret = localStorage.getItem("loginSecret");
        this.#sessionToken = localStorage.getItem("sessionToken");
    }

    get loginSecret(): string | null {
        return this.#loginSecret;
    }

    set loginSecret(loginSecret: string | null) {
        if (loginSecret === null) {
            localStorage.removeItem("loginSecret");
        } else {
            localStorage.setItem("loginSecret", loginSecret);
        }
        this.#loginSecret = loginSecret;
    }

    get sessionToken(): string | null {
        return this.#sessionToken;
    }

    set sessionToken(sessionToken: string | null) {
        if (sessionToken === null) {
            localStorage.removeItem("sessionToken");
        } else {
            localStorage.setItem("sessionToken", sessionToken);
        }
        this.#sessionToken = sessionToken;
    }
}

const credentials = new Credentials();

/** Send an api request.
 *
 * @param method The HTTP method to use.
 * @param path The path to the endpoint, relative to the API URL.
 * @param body The body of the request, if any. Will be JSON encoded.
 * @param authn Whether to include the session token in the request.
 * @returns The HTTP response.
 */
async function endpoint(
    method: "GET" | "POST" | "PATCH" | "DELETE",
    path: string,
    {
        body = null,
        authn = true,
        retryWithNewSession = true,
    }: {
        body?: object | null;
        authn?: boolean;
        retryWithNewSession?: boolean;
    } = {},
): Promise<Response> {
    const headers: Record<string, string> = {};
    if (authn) {
        await ensureLoggedIn(); // This is recursive, but ensureLoggedIn doesn't pass authn=true
        headers.Authorization = `Bearer ${credentials.sessionToken}`;
    }
    if (body !== null) {
        headers["Content-Type"] = "application/json";
    }
    const response = await fetch(`${API_URL}${path}`, {
        method,
        headers,
        body: body && JSON.stringify(body),
    });
    if (response.status === 401 && retryWithNewSession && authn) {
        credentials.sessionToken = null;
        return await endpoint(method, path, {
            body,
            authn,
            retryWithNewSession: false,
        });
    }
    if (!response.ok) {
        const text = await response.text();
        throw new Error(`API Error ${response.status} ${response.statusText}\n${text}`);
    }
    return response;
}

/** Create a new anonymous account, and store the login secret. */
async function createAccount() {
    const response = await endpoint("POST", "/users/me", { authn: false });
    const { login } = await response.json();
    credentials.loginSecret = login;
}

/** Create a new session using a previously stored login secret. Store the session token. */
export async function login() {
    const response = await endpoint("POST", "/sessions", {
        body: {
            method: "secret",
            secret: credentials.loginSecret,
        },
        authn: false,
    });
    const data = await response.json();
    credentials.sessionToken = data.session;
}

async function ensureLoggedIn() {
    if (credentials.sessionToken !== null) {
        return;
    }
    if (credentials.loginSecret !== null) {
        try {
            await login();
            return;
        } catch (e) {
            credentials.loginSecret = null;
            credentials.sessionToken = null;
        }
    }
    await createAccount();
    await login();
}

type Resource<T> = { data: T | undefined; error: object | undefined };

export function useUser(): Resource<User> {
    const fetch = async (path: "/users/me") => {
        return await (await endpoint("GET", path)).json();
    };
    return useSWR("/users/me", fetch);
}

export function useGame(id: number): Resource<Game> {
    type Key = ["/games/:id", number];
    const fetch = async (key: Key) => {
        return await (await endpoint("GET", `/games/${key[1]}`)).json();
    };
    return useSWR<Game, object, Key>(["/games/:id", id], fetch);
}

export function useRecentGames(): Resource<RecentGames> {
    const fetch = async (path: "/games") => {
        return await (await endpoint("GET", path)).json();
    };
    return useSWR("/games", fetch);
}

export function useGenres(): Resource<GenreList> {
    const fetch = async (path: "/genres") => {
        return await (await endpoint("GET", path)).json();
    };
    return useSWR("/genres", fetch);
}

export function useAudio(gameId: number, guesses: number): Resource<HTMLAudioElement> {
    type Key = ["/games/:id/clip", number, number];
    const fetch = async (key: Key) => {
        const blob = await (await endpoint("GET", `/games/${key[1]}/clip`)).blob();
        const url = URL.createObjectURL(blob);
        return new Audio(url);
    };
    return useSWR<HTMLAudioElement, object, Key>(
        ["/games/:id/clip", gameId, guesses],
        fetch,
    );
}

export async function searchTracks(q: string): Promise<TrackSearchResults> {
    const query = new URLSearchParams({ q });
    const path = `/tracks?${query.toString()}`;
    return await (await endpoint("GET", path)).json();
}

function useMutate<Options, Result>(
    key: (_: Options) => Key,
    mutator: (_: Options) => Result,
    { populateCache }: { populateCache: boolean },
) {
    const { mutate } = useSWRConfig();
    const [isLoading, setIsLoading] = useState(false);
    const mutateCb = async (options: Options) => {
        setIsLoading(true);
        const args: MutatorOptions = { populateCache };
        if (populateCache) {
            args.revalidate = false;
        }
        const result = await mutate(key(options), mutator(options), args);
        setIsLoading(false);
        return result;
    };
    return { mutate: mutateCb, isLoading };
}

type UpdateUser = {
    displayName?: string | null;
};

/** Update the current user's display name.
 *
 * @param displayName The new display name, or null to not change it.
 * @returns The updated user account.
 */
async function updateUser({ displayName = null }: UpdateUser): Promise<User> {
    const response = await endpoint("PATCH", "/users/me", {
        body: { display_name: displayName },
    });
    return await response.json();
}

export function useUpdateUser() {
    return useMutate(() => "/users/me", updateUser, { populateCache: true });
}

/** Delete the current user's account. */
async function deleteUser() {
    await endpoint("DELETE", "/users/me");
    credentials.sessionToken = null;
    credentials.loginSecret = null;
}

export function useDeleteUser() {
    return useMutate(() => "/users/me", deleteUser, { populateCache: false });
}

type NewGame = {
    genreId?: number | null;
    daily?: boolean;
    timed?: boolean;
};

/** Create a new game (requires login).
 *
 * @param genreId The genre to pick a song from, or null to pick randomly.
 * @param daily Whether to play the daily game.
 * @param timed Whether to play a timed game mode.
 * @returns The new game.
 *
 * If daily is set, genreId and timed must not be. Will also error if the user
 * has already played the daily game today, or if they already have a game active.
 */
async function newGame({
    genreId = null,
    daily = false,
    timed = false,
}: NewGame = {}): Promise<Game> {
    const response = await endpoint("POST", "/games", {
        body: { genre_id: genreId, daily, timed },
    });
    return await response.json();
}

export function useNewGame() {
    return useMutate(() => "/games", newGame, { populateCache: false });
}

/** Guess a track.
 *
 * @param gameId The ID of the game to submit a guess for.
 * @param trackId The ID of the track to guess, or null to skip a guess.
 * @returns The updated game.
 */
async function newGuess({
    gameId,
    trackId,
}: {
    gameId: number;
    trackId: number | null;
}): Promise<Game> {
    const response = await endpoint("POST", `/games/${gameId}/guesses`, {
        body: { track_id: trackId },
    });
    return await response.json();
}

export function useNewGuess() {
    const key = ({ gameId }: { gameId: number }) => ["/games/:id", gameId];
    return useMutate(key, newGuess, { populateCache: true });
}

/** The current user account, as returned by the API. */
export type User = {
    id: number;
    displayName: string;
    createdAt: string;
};

/** The recent game IDs for the current user. */
export type RecentGames = {
    daily: number | null;
    ongoing: number | null;
};

/** The current game, as returned by the API. */
export type Game = {
    id: number;
    startedAt: Date;
    isDaily: boolean;
    isTimed: boolean;
    genre: Genre | null;
    guesses: Guess[];
    won: boolean | null;
    track: Track | null;
    constants: GameConstants;
};

/** A genre, as returned by the API. */
export type Genre = {
    id: number;
    name: string;
    picture: string;
};

/** A guess within a game, as returned by the API. */
export type Guess = {
    track: Track;
    guessedAt: Date;
};

/** A track, as returned by the API. */
export type Track = {
    id: number;
    title: string;
    link: string;
    artistName: string;
    albumTitle: string;
    albumCover: string;
};

/** Game constants, as returned by the API. */
export type GameConstants = {
    maxGuesses: number;
    musicClipMillis: number[];
    timedUnlockMillis: number[];
};

/** Search results for a track search, as returned by the API. */
export type TrackSearchResults = {
    tracks: Track[];
};

/** A list of genres returned by the API. */
export type GenreList = {
    genres: Genre[];
};
