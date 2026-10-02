<div align="center"> <img src="https://raw.githubusercontent.com/p-tupe/simple-heart-counter/refs/heads/main/logo.png" alt="SHC Image"> </div>

<hr />
<p align="center"><strong>A simple, quick, easy, plug-n-play, fire-n-forget heart-counter for your readers to show their love.</strong></p>
<hr />

## Setup

### Installing the binary

Start `simple-heart-counter` (ideally behind a reverse proxy).

#### Using cargo

```bash
cargo install simple-heart-counter && simple-heart-counter
```

## Connecting to your site

> See `./example/index.html` for full example

Inside your html, just add the following:

```html
<!-- Add an icon with id="shc" and a data-url pointing to your shc server -->
<button id="shc" data-url="https://shc.example.com">&#x2661;&nbsp;</button>

<!-- The script will find #shc, add an on-click event and the count besides it -->
<script src="http://shc.example.com/shc.js" type="text/javascript"></script>
```

## Trial by (local) fire

```bash
# 1. Assuming you have the source
git clone --single-branch --depth=1 https://github.com/p-tupe/simple-heart-counter.git

# 2. Change directory
cd simple-heart-counter

# 3. Start shc server
cargo run

# Then open localhost:3001 and claim your heart! Easy as 1, 2, 3!!!
```

## Extras

`HOST` (=localhost) and `PORT` (=3001) env vars allow changing which addr the server runs on. `RUST_LOG` allows `error`, `warn`, `info` (default) settings. Use 'em like so:

```bash
PORT=8080 RUST_LOG=error simple-heart-counter
```

## How it works

When the script (see `./src/shc.js`) is first loaded, it pulls in the `/count` for current `user` and `url` and appends it by searching `#shc`.

A `user` is identified by `ip:user-agent` from the request header. This does mean that the same user can add mutiple hearts from different browsers/devices. I consider this a feature ;)

A `url` is supplied by the request as `window.location`. I thought of allowing it to be configured but eh, goes beyond the "plug-n-play" doctrine. Feel free to update shc.js as you desire though - MIT license and all that.

When your adoring reader clicks on a heart, the script sends a `/count/increment` request that ups the count and adds the user to the tally. And makes the heart (bleed) red. Your reader cannot touch it again. Atleast, not for giving more love.

If they do click a heart that's already incremented, the script calls `/count/decrement` and that does what the name implies. I added this just for the sake of completeness, I doubt you'll need it.

All this love is saved in a sibling `shc.db` - this path I may make configurable (or not). Make sure to keep it safe by regularly backing it up somewhere else.

And that's that. _Simple_, am I right?

## Roadmap

- Allow custom db path
- Have .shc-unclicked style as well
- ~~Add docker/binaries~~ Used systemd service instead
