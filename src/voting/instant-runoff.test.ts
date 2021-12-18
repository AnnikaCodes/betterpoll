import {instantRunoffCount} from './instant-runoff';

describe('Instant-Runoff Voting', () => {
    test('candidates with >50$ of the vote should win', () => {
        expect(instantRunoffCount(new Set([
            ['A', 'B'],
            ['A', 'C'],
            ['A', 'D'],
            ['A', 'E'],
            ['A', 'C'],
        ]))).toBe('A');

        expect(instantRunoffCount(new Set([
            ['A', 'B'],
            ['A', 'C'],
            ['A', 'D'],
            ['X', 'E'],
            ['X', 'C'],
        ]))).toBe('A');
    });

    test('basic runoff example', () => {
        const dWinsOnRunoff = new Set([
            ['A', 'D'],
            ['C', 'X', 'D'],
            ['B', 'D'],
            ['B', 'D'],
            ['C', 'D'],
            ['X', 'A', 'D'],
            ['A', 'X', 'B', 'D'],
        ]);
        expect(instantRunoffCount(dWinsOnRunoff)).toBe('D');
    });

    test('tie handling', () => {
        const naiveTie = new Set([
            ['A'],
            ['B'],
        ]);
        expect(instantRunoffCount(naiveTie)).toBeNull();

        const noWinner = new Set([
            ['A', 'B'],
            ['A', 'B'],
            ['B', 'C'],
            ['C', 'D'],
            ['D', 'E'],
        ]);
        expect(instantRunoffCount(noWinner)).toBe('A');
    });
});
