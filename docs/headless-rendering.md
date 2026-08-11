# Headless / WebGL rendering notes (Linux)

WebKitGTK (Tauri's Linux webview backend) needs a working GL context — for
both the normal GUI window and the hidden headless-render window — because
3Dmol.js requires WebGL. This is not optional even for `visible: false`
windows.

## Xvfb is required on headless/remote Linux

WebKitGTK requires an active X11 (or Wayland) display connection to
initialize at all, even for invisible windows. On a host with no real display
— a CI runner, a server, or (as encountered during initial development) an
SSH-forwarded X11 display (`ssh -X`) — GLX/DRI3 negotiation with the display
fails and **WebGL context creation fails outright** (`canvas.getContext("webgl")`
returns `null`), regardless of local GPU/Mesa configuration. An SSH-forwarded
display in particular does not proxy GLX/DRI3 correctly, so it behaves like a
headless host even when it "looks" like a display is present (`$DISPLAY` set,
`xdpyinfo` succeeds).

The usual fix is to run against a local **Xvfb** virtual display instead:

```
sudo apt install xvfb
xvfb-run -a crustalline render mol.smi out.png
# or for interactive dev:
xvfb-run -a cargo tauri dev
```

This is a known Tauri/WebKitGTK limitation, not specific to crustalline.
macOS and Windows do not need this.

**Caveat actually observed in the development sandbox this project was
built in**: switching to a local Xvfb display (`Xvfb :99 -screen 0
1280x800x24 +extension GLX +render`) did **not** reliably fix WebGL here
either, despite Xvfb advertising the `GLX` X extension. The main (visible)
window still failed WebGL context creation the same way it did over
SSH-forwarded X11 (`canvas.getContext("webgl")` → `null`). The headless
(invisible) window behaved differently again — sometimes timing out,
sometimes producing a `frame-ready` event whose payload was `data:,` (an
empty/zero-size canvas capture) rather than a real `data:image/png;base64,...`
— nondeterministically across otherwise-identical runs. The most likely
compounding factor: this sandbox's user account is not in the `video`/`render`
group, so `/dev/dri/{card0,renderD128}` (present in the container) are not
actually readable (`ls -la /dev/dri` shows mode `660`), which blocks Mesa's
GBM-backed software path even with `LIBGL_ALWAYS_SOFTWARE=1`, and Xvfb itself
does not implement DRI3 (only classic indirect GLX), which recent WebKitGTK
(EGL-first) does not reliably fall back to. **This was not resolved and is
believed to be specific to this sandbox's device/permission setup, not a
crustalline bug** — none of it points at application code: DOM/CSS render
correctly (confirmed via screenshot — toolbar, input, dark background all
present), only the WebGL canvas specifically fails, and `cargo check` /
`clippy` / the Rust-side `MoleculeState` tests (which don't touch a webview
at all) all pass cleanly. **Verify the actual GUI/PNG output on a machine
with a real GPU (or at minimum, an account with `/dev/dri` render-node access
and a Mesa install known to work with WebKitGTK) before trusting this
sandbox's WebGL results.**

## Env vars — what helps and what doesn't

Encountered while debugging a blank/erroring WebGL canvas in this
environment:

- **`WEBKIT_DISABLE_COMPOSITING_MODE=1`** — do **not** set this to work
  around GL issues. It disables WebKit's compositor outright, which disables
  WebGL entirely (`clearDepth` on a null `_gl` context) — this looks like a
  GL failure but is actually WebGL being turned off.
- **`LIBGL_ALWAYS_SOFTWARE=1`** — forces Mesa's `llvmpipe` software
  rasterizer. Necessary on hosts without a usable GPU, but insufficient on
  its own if the display itself can't negotiate DRI3 (e.g. SSH-forwarded X11)
  — the GL context creation fails before Mesa's software fallback is
  reachable in that case. Works fine once run against a real local X server
  or Xvfb.
- **`WEBKIT_WEBGL_DISABLE_GBM=1`**, **`WEBKIT_DISABLE_DMABUF_RENDERER=1`** —
  skip the GBM/DMA-BUF buffer-sharing path (which needs `/dev/dri` render
  node access) in favor of a plain X11 GL path. Worth trying together with
  `LIBGL_ALWAYS_SOFTWARE=1` if `/dev/dri` isn't accessible to the running
  user (check `ls -la /dev/dri` and whether the user is in the `render`/
  `video` group) — but still requires a display that supports GLX/DRI3 at
  all (i.e. still needs Xvfb on a headless/SSH-forwarded host).
- **`WEBKIT_DISABLE_SANDBOX=1`** — disables WebKit's process sandbox.
  Sometimes needed in containers where sandbox setup itself fails, but was
  not the cause of the WebGL failure encountered here; harmless to set
  defensively.

## Diagnosing a blank/white webview

A webview that paints as a blank white or a plain dark background (matching
only the page's static CSS, no canvas content) with no visible error usually
means a JS error happened before or during `$3Dmol.createViewer(...)`.
`frontend/src/main.ts` proactively probes `canvas.getContext("webgl")` and
surfaces both that result and any thrown error into the `#status` toolbar
element (styled in red) specifically so this is visible via a plain
screenshot, without needing WebKit's remote inspector wired up. If `#status`
shows `webgl: UNAVAILABLE` or an `init error: ... clearDepth` message, the
issue is GL context creation (see above), not application code.

Also note: right after the window's X11 id is mapped, WebKitGTK has not
necessarily finished its first paint yet — an immediate screenshot can catch
a blank frame even when nothing is wrong. Wait for content before concluding
a render failed.

For the **headless render pipeline specifically**, a `frame-ready` payload of
`data:,` (rather than `data:image/png;base64,...`) means `viewer.pngURI()`
captured a zero-size canvas — i.e. WebGL context creation itself succeeded,
but the canvas never got real pixel dimensions. `frontend/src/headless-bridge.ts`
defensively sets the `#viewport` element's `width`/`height` to explicit pixel
values (from the `RenderRequest`) before calling `initViewer`, since an
invisible OS window may never receive a real layout pass and percentage-sized
containers (`width: 100%`) can resolve to `0` in that case. This did not fully
resolve the zero-size/`data:,` symptom in the sandbox described above (results
were nondeterministic even with this fix), but is a reasonable defensive
measure worth keeping regardless, and may be sufficient on a host where the
only issue is layout sizing rather than GL context availability.
