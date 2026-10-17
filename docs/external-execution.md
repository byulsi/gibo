# Execution Handoff

Gibo turns explicit action intent into a portable Execution Brief.
Research, recommendations, target prices, and allocation discussion remain
analytical context until the user requests that handoff.

The requested brief is written as an ordinary project Markdown artifact
outside `gibo/`. It is not Gibo memory, so Record Mode does not gate it,
and its creation does not create a Decision or imply execution.

The brief contains:

- instrument and venue;
- action and size;
- constraints;
- rationale and supporting evidence;
- `as_of` timestamp;
- uncertainty and invalidation conditions.

The user-selected MCP server, app, broker tool, or local program performs the
final effect under the current host's authentication, permission, and
confirmation model.

Local Method elements originally seeded from a Gibo Pack may contribute
analytical methods for preparing a brief. Execution
integration and resulting state live with the selected external capability.
