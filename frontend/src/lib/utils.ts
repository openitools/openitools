import { type ClassValue, clsx } from 'clsx';
import { get } from 'svelte/store';
import { twMerge } from 'tailwind-merge';
import type { Readable } from 'svelte/store';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function when(store: Readable<boolean>, timeoutMs: number = 2147483647): Promise<void> {
  if (get(store)) return Promise.resolve();

  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      unsub();
      reject(new Error('Timeout waiting for store to become true'));
    }, timeoutMs);

    const unsub = store.subscribe((val) => {
      if (val) {
        clearTimeout(timeout);
        unsub();
        resolve();
      }
    });
  });
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
