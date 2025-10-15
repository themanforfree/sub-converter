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

Retrieve a rule from the KV store in Clash rule-provider format.

**URL Parameters:**
- `name`: Rule name

**Example:**
```bash
curl https://your-worker.workers.dev/rules/my-rule
```

**Response:**
- `200 OK`: Returns rule content in Clash rule-provider YAML format
```yaml
payload:
  - DOMAIN-SUFFIX,example.com
  - DOMAIN,exact.example.com
  - IP-CIDR,192.168.0.0/16
```
- `404 Not Found`: Rule not found

**Note:** The endpoint automatically formats stored rules as Clash rule-provider YAML with a `payload` field, making it compatible with Clash's `rule-providers` configuration.

### PUT /rules/:name

Store or update a rule in the KV store.

**URL Parameters:**
- `name`: Rule name

**Query Parameters:**
- `token` (required): Authorization token (must match `RULES_TOKEN` environment variable)

**Body:**
Rule content in one of the following formats:
- Line-separated rules (one rule per line, `#` for comments)
- JSON array of rules

**Example:**
```bash
# Line-separated format
curl -X PUT "https://your-worker.workers.dev/rules/my-rule?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,example.com
DOMAIN,exact.example.com
# This is a comment
IP-CIDR,192.168.0.0/16
EOF

# JSON array format
curl -X PUT "https://your-worker.workers.dev/rules/my-rule?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: application/json" \
  -d '["DOMAIN-SUFFIX,example.com", "DOMAIN,exact.example.com"]'
```

**Response:**
- `200 OK`: Rule stored successfully
- `400 Bad Request`: Invalid request (empty body, missing name)
- `401 Unauthorized`: Missing or invalid authorization token
- `500 Internal Server Error`: Server misconfiguration or KV error

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
    url: https://your-worker.workers.dev/rules/my-rule
    path: ./rules/my-rule.yaml
    interval: 86400
    behavior: classical

rules:
  - RULE-SET,my-custom-rules,Proxies
  - MATCH,DIRECT
```

To update rules dynamically:

```bash
# Upload new rules
curl -X PUT "https://your-worker.workers.dev/rules/my-rule?token=YOUR_RULES_TOKEN" \
  -H "Content-Type: text/plain" \
  --data-binary @- << EOF
DOMAIN-SUFFIX,newsite.com
DOMAIN,another.example.com
IP-CIDR,10.0.0.0/8
EOF

# Clash will fetch the updated rules based on the interval setting
```
