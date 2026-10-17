# Contributing to Gibo

Gibo is licensed under Apache-2.0. By submitting a contribution, you agree
that it is licensed under the same terms. Sign off every commit to certify
that you have the right to submit the work (Developer Certificate of Origin):

```bash
git commit -s
```

Keep the product scoped to project memory, deterministic CLI operations,
data-only Guru Packs, and the derived graph. Execution integrations stay
external. Keep credentials, API keys, portfolio data, private research, and
user memory out of the repository.

Read [AGENTS.md](AGENTS.md) and [docs/architecture.md](docs/architecture.md)
before changing behavior. Run the Rust tests and `claude plugin validate .`
before submitting a change.

Improvements that belong in the shared Rust Runtime or memory contract may be
better contributed to the original
[Guru Maker](https://github.com/monarchjuno/guru-maker) project, under its own
contribution terms.
