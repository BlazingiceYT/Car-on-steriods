# BlazingIce — Bevy/WASM port

## What's working right now
- Your car: loads `assets/car.glb` if it's there, automatically falls back to a
  simple box car if it isn't (so the game never shows a blank screen)
- Full movement physics ported 1:1: acceleration, braking, friction, steering,
  drift, and jump/bump physics (the ground is flat for now, so jumping is
  wired up but dormant until you add real terrain)
- The chase camera
- A placeholder grid so you have somewhere to drive until your real map exists

## What's coming next (say the word and I'll build it)
- AI traffic cars
- Tyre marks
- Your real map, once you're ready to design it

---

## Part 1 — Install Rust (do this once, on your own computer)

Rust doesn't run on GitHub — it has to be compiled on your own machine first.

1. Go to **https://rustup.rs**
2. Download and run the installer for your OS (Windows: `rustup-init.exe`.
   Mac/Linux: paste the command shown on that page into Terminal.)
3. Accept the default options (press Enter / type `1`)
4. Close and reopen your terminal (Command Prompt/PowerShell on Windows,
   Terminal on Mac), then check it worked:
   ```
   rustc --version
   ```
   You should see something like `rustc 1.8x.0 (...)`.
5. Add the ability to compile for web browsers:
   ```
   rustup target add wasm32-unknown-unknown
   ```

## Part 2 — GitHub, without typing git commands

1. Install **GitHub Desktop**: https://desktop.github.com — this gives you
   buttons instead of commands. Sign in with (or create) a GitHub account.
2. In GitHub Desktop: **File → New Repository**. Give it a name and pick a
   folder on your computer — this folder IS the repository from now on.
3. Copy every file from this project (`Cargo.toml`, `src/`, `assets/`,
   `index.html`, `.gitignore`, this `README.md`) into that folder.
4. Back in GitHub Desktop, you'll see the new files listed under "Changes."
   Type a short summary (e.g. "initial car physics"), click **Commit to
   main**, then click **Publish repository** at the top.
5. From now on, any time you change a file: GitHub Desktop shows what
   changed → **Commit** → **Push origin**. That's the whole workflow.

## Part 3 — Adding your car.glb

Drop your `car.glb` file straight into the `assets/` folder in this project,
named exactly `car.glb`. That's it — `src/car.rs` already looks for it there.
(If your file has a different name, open `src/car.rs` and change the
filename inside `.from_asset("car.glb")` to match.)

If the model looks the wrong size, rotated wrong, or floating/sunk into the
ground once it loads, adjust these three constants near the top of
`src/car.rs` and rebuild:
```rust
const CAR_MODEL_SCALE: f32 = 1.0;
const CAR_MODEL_YAW_OFFSET: f32 = std::f32::consts::FRAC_PI_2;
const CAR_MODEL_Y_OFFSET: f32 = 0.0;
```

## Part 4 — Build for the web and publish on GitHub Pages

Open a terminal **inside this project folder** (the one with `Cargo.toml`
in it) and run these one at a time:

```bash
# 1. Quick sanity check — catches typos/errors fast. If this complains,
#    paste me the error before doing anything else.
cargo check

# 2. Full release build, targeting the browser
cargo build --release --target wasm32-unknown-unknown

# 3. Find which wasm-bindgen version Bevy pulled in, and install the exact
#    matching CLI tool — a mismatch here is the most common reason this
#    shows a blank page in the browser.
cargo tree | grep wasm-bindgen
cargo install wasm-bindgen-cli --version <the version number you just saw>

# 4. Generate the JS + .wasm the browser actually loads
wasm-bindgen --target web --no-typescript \
  --out-dir web --out-name blazingice_bevy \
  target/wasm32-unknown-unknown/release/blazingice_bevy.wasm

# 5. Copy the files the page needs alongside what wasm-bindgen just made
cp index.html web/
cp -r assets web/
```

You now have a `web/` folder containing everything needed to run the game as
a static website. In GitHub Desktop: commit and push it (same as Part 2,
step 5).

Then, on **github.com**, open your repository → **Settings → Pages** → under
"Build and deployment," choose **Deploy from a branch**, pick your branch,
and set the folder to **`/web`** → **Save**. GitHub gives you a URL like
`https://yourname.github.io/reponame/` — usually live within a minute or two.

**Every time you change the Rust code**, repeat steps 1–5 above and push the
updated `web/` folder again — it isn't automatic yet. (A GitHub Actions
workflow can automate this later; ask me once you're comfortable with the
manual flow and I'll set it up.)

## If something won't compile

Bevy's API changes with every release, and I can't run a Rust compiler
myself to double-check this against the exact version you get — `cargo check`
on your machine is the real test. This copy already has the fixes for the
Bevy 0.19-specific renames we hit last round (`SceneRoot` → `WorldAssetRoot`,
`Scene` → `WorldAsset`, `AmbientLight` → `GlobalAmbientLight` resource,
`Msaa` moved to a per-camera component, `shadows_enabled` →
`shadow_maps_enabled`), so `cargo check --target wasm32-unknown-unknown`
should now get much further. Paste me the exact error if it doesn't, and
I'll fix it.

**Important:** always run `cargo check --target wasm32-unknown-unknown`
(not plain `cargo check`) — without `--target wasm32-unknown-unknown` it
tries to build for the Codespace's own Linux machine and fails on an
unrelated missing `wayland-client` library that has nothing to do with your
code.
