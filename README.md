# bettervote
## API
Something along these lines (poll IDs should be integers or hexstrings or something):
- `POST /api/<pollid>/vote` with candidate choices to vote
    - provided data should be JSON of the form `{"choices":[]}`, where the `choices` key is an array of candidate strings
    - response will be `{"success": true}` or equivalent JSON if the vote succeeds, and `{"success": false, "error": <errorstring>}` or equivalent if it fails (where `<errorstring>` is a string explaining the error that occured)
- `GET /api/:pollid` to get info about a poll
- `POST /api/:pollid/create` (maybe `PUT`?) with candidates + voting system + expiry time/offset to create a poll

A frontend will also need to be worked out.
That can probably go into a separate repository, but should access this API from the browser
and have a pretty interface.

TODO: look into different frontend frameworks (Vue? Svelte? Flutter?).

## Database
Implements a way to store information needed for the website.

- Postgres abstraction
    - should make it easy to switch between databases in case we need to switch to AWS/MySQL/etc
    - should store individual polls (creation time, expiry time, candidates, voting system)
- should store votes associated with a given poll (candidate choices, some way to identify the voter)
    - consider making the voter identification generic somehow???
        - may be IP address/fingerprint/both/nothing

## Voting algorithms
Use [`tallystick`](https://crates.io/crate/tallystick) to get the Schulze method etc.