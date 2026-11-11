/** A loose shape check to enable the "Continue" button; the API does the real validation. */
export const looksLikeEmail = (value: string) => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value.trim());
