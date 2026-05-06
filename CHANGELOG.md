# Changelog

## [Unreleased] — ~30% complete

### Done
- Workspace and crate scaffolding (`Cargo.toml`, `contracts/tipjar/Cargo.toml`)
- `DataKey` storage enum (`Token`, `CreatorBalance`, `CreatorTotal`)
- `init(token)` — one-time token address configuration with double-init guard
- Unit tests: `test_init`, `test_init_twice_panics`

### Pending
- `tip(sender, creator, amount)` — token escrow transfer and balance tracking
- `get_total_tips(creator)` — read cumulative historical total
- `withdraw(creator)` — release escrowed balance to creator
- On-chain events for `tip` and `withdraw`
- Full unit test suite (`test_tip_and_totals`, `test_withdraw`, `test_invalid_tip_amount`)
- Testnet deploy script (`scripts/deploy.sh`)
