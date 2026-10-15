# OpenIntents plugin for Claude Code

Adds the hosted OpenIntents MCP server and the `openintents` skill to Claude
Code, so you can ask for purchases in plain language ("get me a flat white
from the Blue Bottle on 5th, pickup 8:45, max $10").

```text
/plugin marketplace add tinyhumansai/openintents
/plugin install openintents@openintents
```

The server takes an API key: make one at
<https://openintents.io/dashboard/api-keys> (start with a test key,
`oi_test_...`: test intents run end to end but never charge) and set it as
`OPENINTENTS_API_KEY` before starting Claude Code. The plugin's server sends
it as `Authorization: Bearer`. Prefer a one-off server without the plugin?

```bash
claude mcp add --transport http openintents https://api.openintents.io/mcp \
  --header "Authorization: Bearer $OPENINTENTS_API_KEY"
```

See <https://openintents.io/docs/mcp>.
