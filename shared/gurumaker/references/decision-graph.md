# Decision Graph contract

`gurumaker graph` scans only local elements under `guru-maker/` and writes
`guru-maker/decision-graph.html`.

Valid analytical edges are:

```yaml
# Evidence -> earlier Evidence
updates:
  - evidence:earlier-observation

# Decision -> reusable memory, Evidence, or earlier Decision
uses:
  - wiki:advanced-packaging
  - lens:quality-growth
  - method:channel-review
supports:
  - evidence:example-corp:q2-renewal-rate
contradicts:
  - decision:example-corp:prior-review
```

Evidence source URLs become source nodes. Pack source checkouts and Pack nodes
never enter the graph. The graph does not infer relationships from prose or
ordinary Markdown links. Missing or wrong-kind targets are reported rather
than invented.

The HTML is a disposable navigation view, not memory or authority. Edit local
elements and regenerate it. Surface nonzero unresolved or skipped counts.
