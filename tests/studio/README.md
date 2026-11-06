These focused App tests compile the Rust schema with Studio's existing compiler and exercise the shared editor, link-return and draft-restoration owners against that schema. They use the dependencies already installed in a sibling Studio checkout.

Run from the Tax checkout:

```sh
export LIBERTAS_STUDIO=/path/to/libertas/vscode
node "$LIBERTAS_STUDIO/node_modules/jest/bin/jest.js" --config tests/studio/jest.config.cjs --maxWorkers=2
```

The first split remains an incomplete local editor until valid parts are supplied. The tests submit no incomplete protocol message. Native source-save regressions separately cover accepted identities, relational validation and stale-revision refusal.
