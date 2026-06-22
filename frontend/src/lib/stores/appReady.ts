import { writable } from 'svelte/store';

export const appReady = writable<boolean>(false);