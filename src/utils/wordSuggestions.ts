/** Single English token, at least two letters. Phrases and Chinese stay out of the local list. */
const SUGGESTABLE = /^[A-Za-z][A-Za-z'-]{1,31}$/;

export function canSuggestQuery(value: string): boolean {
  return SUGGESTABLE.test(value.trim());
}
