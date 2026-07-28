# External Execution Brief

Create an Execution Brief only when the user explicitly asks to act or asks
for the brief. It is a copyable external Markdown document, not an order and
not an execution mechanism.

Write the requested brief as an ordinary project Markdown artifact outside
`guru-maker/`. Use the user's path when supplied; otherwise choose a concise,
descriptive, collision-safe filename. Guru Mode governs Guru memory, not this
requested handoff artifact. Creating the brief does not create a Decision,
authorize execution, or imply that any external action occurred.

```markdown
# Execution Brief — Example Corp

**Status:** Draft for external review only. Not an order; no execution effect.

## Requested action

- Instrument: An unambiguous identifier.
- Venue: The user-selected venue, or explicitly unresolved.
- Action: The user's explicit buy, sell, or rebalance request.
- Size: The user-provided or approved amount and units.
- Constraints: Only user-provided or approved timing, price, and order limits.

## Rationale and evidence

- [Related decision](../decision/example-corp-entry.md)
- Source-bound evidence supporting the request.

## Point-in-time context

- As of: The observation timestamp for material market or source information.
- Uncertainty: Material unknowns and unresolved items.

## Invalidation conditions

- Conditions that would make the rationale or requested action no longer valid.

## Preconditions and checks

- User confirmation and any external account, policy, tax, liquidity, or
  execution requirements.
```

Do not invent the instrument, venue, action, size, constraints, rationale,
evidence, as-of context, uncertainty, invalidation conditions, account details,
approval status, or order parameters. Keep credentials out of the brief. If
intent is ambiguous, continue analysis or ask for clarification instead.
