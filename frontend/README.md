HearthGlow frontend

This is a tiny TypeScript-based static frontend shipped under `frontend/dist` and served by the Rust Axum server.

Structure:
- `frontend/src` - TypeScript source (not required at runtime)
- `frontend/dist` - files served to browsers (index.html, styles.css, app.js)

No build step is required to run the demo because compiled JS is included in `frontend/dist`.

To iterate on the TypeScript source, compile it with your preferred toolchain (esbuild, tsc, etc.) and place the output into `frontend/dist/app.js`.
