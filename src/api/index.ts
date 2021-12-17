/**
 * API endpoints for bettervote.
 *
 * Something along these lines (poll IDs should be integers or hexstrings or something):
 * - POST /api/${pollid}/vote with candidate choices to vote
 * - GET /api/${pollid} to get info about a poll
 * - POST (or PUT?) /api/${pollid}/create with candidates + voting system + expiry time/offset to create a poll
 *
 * A frontend will also need to be worked out.
 * That can probably go into a separate repository, but should access this API from the browser
 * and have a pretty interface.
 *
 * TODO: look into different frontend frameworks (Vue? Svelte? Flutter?).
 */
