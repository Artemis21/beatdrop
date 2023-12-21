/** @file Low level API endpoint wrappers, including storage of credentials. */

const API_URL = "/api";

/** Singleton class which handles storing credentials in local storage. */
class Credentials {
    /** The login secret, used to authenticate an anonymous account to create sessions. */
    #loginSecret = localStorage.getItem("loginSecret");

    /** The session token, used to authenticate requests. */
    #sessionToken = localStorage.getItem("sessionToken");

    constructor() {
        this.#loginSecret = localStorage.getItem("loginSecret");
        this.#sessionToken = localStorage.getItem("password");
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

/** Check whether the user has an existing account. */
export const hasAccount = () => credentials.loginSecret !== null;

/** Clear the user's login secret, forgetting their account. */
export const clearAccount = () => {
    credentials.loginSecret = null;
    credentials.sessionToken = null;
};

/** Check whether the user is logged in. */
export const isLoggedIn = () => credentials.sessionToken !== null;

/** Clear the user's session token, logging them out. */
export const clearSession = () => credentials.sessionToken = null;

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
    }: {
        body?: object | null;
        authn?: boolean;
    } = {},
): Promise<Response> {
    const headers: Record<string, string> = {};
    if (authn) {
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
    if (!response.ok) {
        const text = await response.text();
        throw new Error(
            `API Error ${response.status} ${response.statusText}\n${text}`,
        );
    }
    return response;
}

/** The current account, as returned by the API. */
export type Account = {
    id: number;
    displayName: string;
    createdAt: Date;
};

/** Parse an API response as an account. */
function intoAccount(data: any): Account {
    return {
        id: data.id,
        displayName: data.display_name,
        createdAt: new Date(data.created_at),
    };
}

/** Create a new anonymous account, and store the login secret. */
export async function createAccount() {
    const response = await endpoint("POST", "/account/me", { authn: false });
    const data = await response.json();
    credentials.loginSecret = data.login;
}

/** Create a new session using a previously stored login secret. Store the session token. */
export async function login() {
    const response = await endpoint("POST", "/session/secret", {
        body: { login: credentials.loginSecret },
        authn: false,
    });
    const data = await response.json();
    credentials.sessionToken = data.session;
}

/** Get the current account (requires login). */
export async function getAccount(): Promise<Account> {
    const response = await endpoint("GET", "/account/me");
    return intoAccount(await response.json());
}

/** Update the current account's display name (requires login).
 *
 * @param displayName The new display name, or null to not change it.
 * @returns The updated account.
 */
export async function updateAccount({
    displayName = null,
}: {
    displayName?: string | null;
}): Promise<Account> {
    const response = await endpoint("PATCH", "/account/me", {
        body: { display_name: displayName },
    });
    return intoAccount(await response.json());
}

/** Delete the current account (requires login). */
export async function deleteAccount() {
    await endpoint("DELETE", "/account/me");
    credentials.sessionToken = null;
    credentials.loginSecret = null;
}

/** The current game, as returned by the API. */
export type Game = {
    startedAt: Date;
    isDaily: boolean;
    isTimed: boolean;
    genre: Genre | null;
    guesses: Guess[];
    unlockedSeconds: number;
    won: boolean | null;
    track: Track | null;
};

/** Parse an API response as a game. */
function intoGame(data: any): Game {
    return {
        startedAt: new Date(data.started_at),
        isDaily: data.is_daily,
        isTimed: data.is_timed,
        genre: data.genre && intoGenre(data.genre),
        guesses: data.guesses.map(intoGuess),
        unlockedSeconds: data.unlocked_seconds,
        won: data.won,
        track: data.track && intoTrack(data.track),
    };
}

/** A genre, as returned by the API. */
export type Genre = {
    id: number;
    name: string;
    picture: string;
};

/** Parse data from an API response as a genre. */
function intoGenre(data: any): Genre {
    return {
        id: data.id,
        name: data.name,
        picture: data.picture,
    };
}

/** A guess within a game, as returned by the API. */
export type Guess = {
    trackId: number;
    guessedAt: Date;
};

/** Parse data from an API response as a guess. */
function intoGuess(data: any): Guess {
    return {
        trackId: data.track_id,
        guessedAt: new Date(data.guessed_at),
    };
}

/** A track, as returned by the API. */
export type Track = {
    id: number;
    title: string;
    link: FunctionStringCallback;
    artistName: string;
    albumName: string;
    albumCover: string;
};

/** Parse data from an API response as a track. */
function intoTrack(data: any): Track {
    return {
        id: data.id,
        title: data.title,
        link: data.link,
        artistName: data.artist_name,
        albumName: data.album_name,
        albumCover: data.album_cover,
    };
}

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
export async function newGame({
    genreId = null,
    daily = false,
    timed = false,
}: {
    genreId?: number | null;
    daily?: boolean;
    timed?: boolean;
} = {}): Promise<Game> {
    const response = await endpoint("POST", "/game", {
        body: { genre_id: genreId, daily, timed },
    });
    return intoGame(await response.json());
}

/** Get the current game (requires login). */
export async function getActiveGame(): Promise<Game> {
    const response = await endpoint("GET", "/game");
    return intoGame(await response.json());
}

/** Guess a track (requires login).
 *
 * @param trackId The ID of the track to guess, or null to skip a guess.
 * @returns The updated game.
 */
export async function newGuess(trackId: number | null): Promise<Game> {
    const response = await endpoint("POST", "/game/guess", {
        body: { track_id: trackId },
    });
    return intoGame(await response.json());
}

/** Get an audio clip for the current game (requires login).
 *
 * @param seek The number of seconds to seek into the clip, or null for the full clip.
 * @returns The track clip as a WAV blob.
 */
export async function gameClip(seek: number | null = null): Promise<Blob> {
    let path = "/game/clip";
    if (seek !== null) {
        path += `?seek=${seek}`;
    }
    const response = await endpoint("GET", path);
    return await response.blob();
}
