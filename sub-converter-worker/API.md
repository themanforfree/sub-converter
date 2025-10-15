# Cloudflare Worker API

This document describes the HTTP API endpoints provided by the sub-converter Cloudflare Worker.

## Endpoints

### GET /profile

Convert subscription sources to target format with optional template.

**Query Parameters:**
- `origin_url` (required): Source subscription URL
- `origin_kind` (optional): Source type - `auto`, `clash`, `singbox`, or `uri` (default: `auto`)
- `target_kind` (required): Target format - `clash` or `singbox`
- `encoding` (optional): Output encoding - `json` or `yaml`
- `template_b64` (optional): Base64-encoded template content
- `template_url` (optional): Template URL (http/https)
- `template_name` (optional): Template name stored in R2 bucket
- `token` (required): Authorization token (must match `PROFILE_TOKEN` environment variable)

**Example:**
```bash
curl "https://your-worker.workers.dev/profile?origin_url=https://example.com/sub&target_kind=clash&token=YOUR_TOKEN"
```

### GET /template/:name

Retrieve a template from the R2 bucket.

**URL Parameters:**
- `name`: Template name

**Example:**
```bash
curl https://your-worker.workers.dev/template/my-template
```

### PUT /template/:name

Upload a template to the R2 bucket.

**URL Parameters:**
- `name`: Template name

**Query Parameters:**
- `token` (required): Authorization token (must match `TEMPLATE_TOKEN` environment variable)

**Body:**
Template content (YAML or JSON)

**Example:**
```bash
curl -X PUT "https://your-worker.workers.dev/template/my-template?token=YOUR_TEMPLATE_TOKEN" \
  -H "Content-Type: application/yaml" \
  --data-binary @template.yaml
```

**Response:**
- `200 OK`: Template uploaded successfully
- `400 Bad Request`: Invalid request (empty body, missing name)
- `401 Unauthorized`: Missing or invalid authorization token
- `500 Internal Server Error`: Server misconfiguration or R2 error

### GET /rules

List all available rules in the KV store.

**Example:**
```bash
curl https://your-worker.workers.dev/rules
```

**Response:**
- `200 OK`: Returns JSON array of rule names
```json
{
  "rules": ["my-rule", "another-rule"]
}
```

### GET /rules/:name

Retrieve an entire rule set from the KV store in Clash rule-provider format.

**URL Parameters:**
- `name`: Rule set name

**Example:**
```bash
curl https://your-worker.workers.dev/rules/my-rules
```

**Response:**
- `200 OK`: Returns rule set in Clash rule-provider YAML format
```yaml
payload:
  - DOMAIN-SUFFIX,example.com
  - DOMAIN,exact.example.com
  - IP-CIDR,192.168.0.0/16
```
- `404 Not Found`: Rule set not found

**Note:** The endpoint automatically formats stored rules as Clash rule-provider YAML with a `payload` field, making it compatible with Clash's `rule-providers` configuration.

### PUT /rules/:name

Replace an entire rule set in the KV store.

**URL Parameters:**
- `name`: Rule set name

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Body:**
Rule content in one of the following formats:
- Line-separated rules (one rule per line, `#` for comments)
- JSON array of rules

**Example:**
```bash
# Line-separated format
curl -X PUT "https://your-worker.workers.dev/rules/my-rules?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,example.com
DOMAIN,exact.example.com
# This is a comment
IP-CIDR,192.168.0.0/16
EOF

# JSON array format
curl -X PUT "https://your-worker.workers.dev/rules/my-rules?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: application/json" \
  -d '["DOMAIN-SUFFIX,example.com", "DOMAIN,exact.example.com"]'
```

**Response:**
- `200 OK`: Rule set replaced successfully
- `400 Bad Request`: Invalid request (empty body, missing name)
- `401 Unauthorized`: Missing or invalid authorization token
- `500 Internal Server Error`: Server misconfiguration or KV error

### POST /rules/:name/rules

Add rules to an existing rule set (or create a new one if it doesn't exist).

**URL Parameters:**
- `name`: Rule set name

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Body:**
Rules to add (line-separated or JSON array format)

**Example:**
```bash
# Add new rules to existing set
curl -X POST "https://your-worker.workers.dev/rules/my-rules/rules?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,newsite.com
IP-CIDR,10.0.0.0/8
EOF
```

**Response:**
- `200 OK`: Rules added successfully
- `400 Bad Request`: Invalid request
- `401 Unauthorized`: Missing or invalid authorization token

### PUT /rules/:name/rules/:index

Update a specific rule in a rule set by its index.

**URL Parameters:**
- `name`: Rule set name
- `index`: Rule index (0-based)

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Body:**
New rule content (plain text)

**Example:**
```bash
# Update rule at index 0
curl -X PUT "https://your-worker.workers.dev/rules/my-rules/rules/0?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  -d "DOMAIN-SUFFIX,updated.com"
```

**Response:**
- `200 OK`: Rule updated successfully
- `400 Bad Request`: Invalid index or empty content
- `401 Unauthorized`: Missing or invalid authorization token
- `404 Not Found`: Rule set not found

### DELETE /rules/:name/rules/:index

Delete a specific rule from a rule set by its index.

**URL Parameters:**
- `name`: Rule set name
- `index`: Rule index (0-based)

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Example:**
```bash
# Delete rule at index 2
curl -X DELETE "https://your-worker.workers.dev/rules/my-rules/rules/2?token=YOUR_RULES_TOKEN"
```

**Response:**
- `200 OK`: Rule deleted successfully
- `400 Bad Request`: Invalid index or cannot delete last rule
- `401 Unauthorized`: Missing or invalid authorization token
- `404 Not Found`: Rule set not found

### DELETE /rules/:name

Delete an entire rule set.

**URL Parameters:**
- `name`: Rule set name

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Example:**
```bash
curl -X DELETE "https://your-worker.workers.dev/rules/my-rules?token=YOUR_RULES_TOKEN"
```

**Response:**
- `200 OK`: Rule set deleted successfully
- `401 Unauthorized`: Missing or invalid authorization token

## Environment Variables

The worker requires the following environment variables:

- `PROFILE_TOKEN`: Token for authenticating profile conversion requests
- `TEMPLATE_TOKEN`: Token for authenticating template upload requests
- `RULES_TOKEN`: Token for authenticating rules upload requests

## R2 Bucket Configuration

The worker uses an R2 bucket binding named `TEMPLATE` for storing templates. Configure this in `wrangler.toml`:

```toml
[[r2_buckets]]
binding = 'TEMPLATE'
bucket_name = 'template'
preview_bucket_name = 'template-dev'
```

## KV Namespace Configuration

The worker uses a KV namespace binding named `RULES` for storing rules. Configure this in `wrangler.toml`:

```toml
[[kv_namespaces]]
binding = 'RULES'
id = 'rules'
preview_id = 'rules-dev'
```

## Using Rules with Clash

The `/rules/:name` endpoint returns rules in Clash rule-provider format. You can reference them in your Clash configuration:

```yaml
rule-providers:
  my-custom-rules:
    type: http
    url: https://your-worker.workers.dev/rules/my-custom-rules
    path: ./rules/my-custom-rules.yaml
    interval: 86400
    behavior: classical

rules:
  - RULE-SET,my-custom-rules,Proxies
  - MATCH,DIRECT
```

### Managing Rules Dynamically

**Replace entire rule set:**
```bash
curl -X PUT "https://your-worker.workers.dev/rules/my-custom-rules?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,example.com
DOMAIN,exact.example.com
IP-CIDR,192.168.0.0/16
EOF
```

**Add rules to existing set:**
```bash
curl -X POST "https://your-worker.workers.dev/rules/my-custom-rules/rules?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,newsite.com
IP-CIDR,10.0.0.0/8
EOF
```

**Update a specific rule (by index):**
```bash
# Update rule at index 0
curl -X PUT "https://your-worker.workers.dev/rules/my-custom-rules/rules/0?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  -d "DOMAIN-SUFFIX,updated.com"
```

**Delete a specific rule (by index):**
```bash
# Delete rule at index 1
curl -X DELETE "https://your-worker.workers.dev/rules/my-custom-rules/rules/1?token=YOUR_RULES_TOKEN"
```

**Delete entire rule set:**
```bash
curl -X DELETE "https://your-worker.workers.dev/rules/my-custom-rules?token=YOUR_RULES_TOKEN"
```

Clash will fetch the updated rules based on the interval setting in your configuration.
