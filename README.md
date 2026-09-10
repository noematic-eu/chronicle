# Chronicle

A campaign engine: briefing → petitions → set-piece → debrief → lineage.
Stories live in a separate pack (`chronicle-stories`). This repo is the player.

```
.md packs  →  chronicle-dsl  →  chronicle.ir/v1  →  chronicle play
```

## Play

```bash
cargo run -- play ../chronicle-stories/france/play
cargo run -- play ../chronicle-stories/peru/play
```

Keys: j/k, Enter, n notes, ? help, q quit.

Saves: `instance/chronicle/<france|peru>/lineage.json`.

Override the packs root with `CHRONICLE_STORIES=/path/to/chronicle-stories`.

## Layout

| Crate | Role |
|---|---|
| `chronicle` | TUI binary |
| `chronicle-dsl` | Markdown + YAML fence → IR |
| `chronicle-engine` | chapter mode machine (no I/O) |

## License

[MIT](./LICENSE). Packs you write stay yours.
