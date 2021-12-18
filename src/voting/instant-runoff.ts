/**
 * Implementation of the Instant-Runoff Voting (IRV) ranked-choice voting system.
 *
 * @see https://en.wikipedia.org/wiki/Instant-runoff_voting
 * @see https://www.youtube.com/watch?v=3Y3jE3B8HsE
 */

import type {Vote} from '.';

/**
 * Tallies votes.
 *
 * Candidates in `exclude` will be excluded, and voters' next-highest choice will be counted instead
 */
function tallyVotes<T>(
    votes: Iterable<Vote<T>>,
    exclude: Set<T>
): {tally: Map<T, number>; threshold: number} {
    const tally = new Map();
    let voteCount = 0;
    for (const vote of votes) {
        let hasBeenCounted = false;
        for (const choice of vote) {
            const candidates = Array.isArray(choice) ? choice : [choice];

            let allExcluded = true;
            for (const candidate of candidates) {
                if (exclude.has(candidate)) continue;
                const count = tally.get(candidate) || 0;
                tally.set(candidate, count + 1);
                hasBeenCounted = true;
                allExcluded = false;
            }
            if (allExcluded) continue; // move to the next choice
        }
        if (hasBeenCounted) voteCount++;
    }

    return {tally, threshold: voteCount / 2};
}

/**
 * Finds the winner.
 *
 * This will consume `election`.
 *
 * TODO: fuzz this & make sure theres no way to provoke an infinite loop
 *
 * TODO: figure out what to do with Set { ['A', 'B'], ['B', 'A'] }
 * @returns null if no majority winner is found
 */
export function instantRunoffCount<T extends keyof any>(votes: Set<Vote<T>>): T | null {
    let {tally, threshold} = tallyVotes<T>(votes, new Set());
    const numCandidates = tally.size;
    const eliminated: Set<T> = new Set();

    while (tally.size > 1) {
        let leastPopular: {candidate: T; votes: number} | null = null;

        for (const [candidate, numVotes] of tally) {
            if (numVotes > threshold) return candidate;

            if (leastPopular === null || numVotes < leastPopular.votes) {
                leastPopular = {candidate, votes: numVotes};
            }
        }

        // Eliminate least popular candidate
        if (!leastPopular) throw new Error(`No least popular candidate found`); // should never happen
        eliminated.add(leastPopular.candidate);
        if (eliminated.size === numCandidates) return null;
        ({tally, threshold} = tallyVotes<T>(votes, eliminated));

        console.log(tally, eliminated, threshold, eliminated.size, numCandidates);
    }

    // No majority found :(
    return null;
}

