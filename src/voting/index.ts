/**
 * General voting code common to all ranked-choice voting systems implemented here.
 */

/**
 * Represents an individual's vote.
 *
 * Index 0 represents their #1 choice(s), index 1 represents their #2 choice(s), etc.
 *
 * Elements can be a `T` (one candidate was ranked at this choice),
 * or a `T[]` (multiple candidates were ranked at this choice).
 *
 * For example, in an election with five candidates (Alice, Bob, Charlie, David, and Eve):
 * - A user who votes Alice #1 and Bob #2 would have a `Votes<string>` of ['Alice', 'Bob'].
 * - A user who votes Alice and Bob jointly #1 and Eve #2 would have a `Votes<string>` of [['Alice', 'Bob'], 'Eve'].
 * - A user who votes for all candidates might have something like ['Eve', ['David', 'Alice'], 'Charlie', 'Bob'].
 */
export type Vote<T> = (T | T[])[];

/**
 * Gets winners.
 * The returned array must have a length equal to the minimum of `numWinners` and the number of candidates.
 */
export type VoteCounter<T extends keyof any> = (votes: Iterable<Vote<T>>, numWinners: number) => T[];
