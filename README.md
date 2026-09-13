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
cargo run -- play ../chronicle-stories/silence/play
cargo run -- lint ../chronicle-stories/france/play
```

Keys: j/k, Enter, n notes, ? help, q quit.

Saves: `instance/chronicle/<france|peru|silence>/lineage.json`.

`heritage` on a choice is a union (it never wipes the cave). `next` must name a chapter in the pack or `lint` fails. `cheval_si_chevaux: true` (France 1.1) is the only place a `chevaux` stock also adds the `cheval` relic.

Override the packs root with `CHRONICLE_STORIES=/path/to/chronicle-stories`.

## Layout

| Crate | Role |
|---|---|
| `chronicle` | TUI binary |
| `chronicle-dsl` | Markdown + YAML fence → IR |
| `chronicle-engine` | chapter mode machine (no I/O) |

## License

[MIT](./LICENSE). Packs you write stay yours.
