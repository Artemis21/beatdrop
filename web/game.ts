/** @file Manage the user's ongoing game. */
import * as api from "./api";
import account from "./account";

export default class Game {
    #data: api.Game | null = null;
    #lastRefresh: Date | null = null;

    async #refresh() {
        await account.login();
        this.#data = await api.getActiveGame();
    }

    async #ensureFresh() {
        const MAX_AGE = 1000 * 60; // refresh at least every minute
        if (
            this.#lastRefresh === null ||
            this.#lastRefresh.getTime() < Date.now() - MAX_AGE
        ) {
            await this.#refresh();
            this.#lastRefresh = new Date();
        }
    }

    async startedAt(): Promise<Date | undefined> {
        await this.#ensureFresh();
        return this.#data?.startedAt;
    }

    async isDaily(): Promise<boolean | undefined> {
        await this.#ensureFresh();
        return this.#data?.isDaily;
    }

    async isTimed(): Promise<boolean | undefined> {
        await this.#ensureFresh();
        return this.#data?.isTimed;
    }

    async genre(): Promise<api.Genre | null | undefined> {
        await this.#ensureFresh();
        return this.#data?.genre;
    }

    async guesses(): Promise<api.Guess[] | undefined> {
        await this.#ensureFresh();
        return this.#data?.guesses;
    }

    async unlockedSeconds(): Promise<number | undefined> {
        await this.#ensureFresh();
        return this.#data?.unlockedSeconds;
    }

    async won(): Promise<boolean | null | undefined> {
        await this.#ensureFresh();
        return this.#data?.won;
    }

    async track(): Promise<api.Track | null | undefined> {
        await this.#ensureFresh();
        return this.#data?.track;
    }

    async guess(trackId: number | null) {
        await account.login();
        this.#data = await api.newGuess(trackId);
    }

    async audio(seek: number): Promise<Blob> {
        await account.login();
        return await api.gameClip(seek);
    }
}
