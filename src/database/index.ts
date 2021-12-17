/**
 * Implements a way to store information needed for the website.
 *
 * - Postgres abstraction
 *   - should make it easy to switch between databases in case we need to switch to AWS/MySQL/etc
 * - should store individual polls (creation time, expiry time, candidates, voting system)
 * - should store votes associated with a given poll (candidate choices, some way to identify the voter)
 *   - consider making the voter identification generic somehow???
 *     - may be IP address/fingerprint/both/nothing
 *     - could share ID code with Krytis
 */
