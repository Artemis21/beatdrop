/** @file Manage the user's account and session. */
import * as api from "./api";

export class Account {
    #data: api.Account | null = null;
    #lastRefresh: Date | null = null;

    async #refresh() {
        if (api.isLoggedIn()) {
            try {
                this.#data = await api.getAccount();
                return;
            } catch (e) {
                api.clearSession();
            }
        }
        if (api.hasAccount()) {
            try {
                await api.login();
            } catch (e) {
                api.clearAccount();
            } finally {
                this.#data = await api.getAccount();
                return;
            }
        }
        await api.createAccount();
        await api.login();
        this.#data = await api.getAccount();
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

    async login() {
        this.#ensureFresh();
    }

    async logout() {
        api.clearSession();
        this.#data = null;
        this.#lastRefresh = null;
    }

    async delete() {
        await api.deleteAccount();
        this.logout();
    }

    async id(): Promise<number> {
        await this.#ensureFresh();
        return this.#data!.id;
    }

    async displayName(): Promise<string> {
        await this.#ensureFresh();
        return this.#data!.displayName;
    }

    async setDisplayName(displayName: string) {
        this.#data = await api.updateAccount({ displayName });
    }

    async createdAt(): Promise<Date> {
        await this.#ensureFresh();
        return this.#data!.createdAt;
    }
}

export const account = new Account();
export default account;
