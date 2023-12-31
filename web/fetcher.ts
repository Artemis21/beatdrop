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
        await ensureLoggedIn();  // This is recursive, but ensureLoggedIn doesn't pass authn=true
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
        throw new Error(
            `API Error ${response.status} ${response.statusText}\n${text}`,
        );
    }
    return response;
}

/** Create a new anonymous account, and store the login secret. */
async function createAccount() {
    const response = await endpoint("POST", "/account/me", { authn: false });
    const { login } = await response.json();
    credentials.loginSecret = login;
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

export async function fetcher(key: "/account/me"): Promise<Account>;
export async function fetcher(key: "/game" | "/game/daily"): Promise<Game | null>;
export async function fetcher(key: `/game/clip`): Promise<Blob>;
export async function fetcher(key: string): Promise<object | null> {
    const response = await endpoint("GET", key);
    if (response.headers.get("Content-Type") === "audio/wav") {
        return await response.blob();
    }
    return await response.json();
}

/** Update the current account's display name.
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
    return await response.json();
}

/** Delete the current account. */
export async function deleteAccount() {
    await endpoint("DELETE", "/account/me");
    credentials.sessionToken = null;
    credentials.loginSecret = null;
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
    return await response.json();
}

/** Guess a track.
 *
 * @param trackId The ID of the track to guess, or null to skip a guess.
 * @returns The updated game.
 */
export async function newGuess(trackId: number | null): Promise<Game> {
    const response = await endpoint("POST", "/game/guess", {
        body: { track_id: trackId },
    });
    return await response.json();
}

/** The current account, as returned by the API. */
export type Account = {
    id: number;
    displayName: string;
    createdAt: string;
};

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

/** A genre, as returned by the API. */
export type Genre = {
    id: number;
    name: string;
    picture: string;
};

/** A guess within a game, as returned by the API. */
export type Guess = {
    trackId: number;
    guessedAt: Date;
};

/** A track, as returned by the API. */
export type Track = {
    id: number;
    title: string;
    link: FunctionStringCallback;
    artistName: string;
    albumName: string;
    albumCover: string;
};
