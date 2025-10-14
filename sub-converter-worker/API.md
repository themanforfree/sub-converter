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

**Headers:**
- `Authorization: Bearer <token>` (required): Must match `TEMPLATE_TOKEN` environment variable

**Body:**
Template content (YAML or JSON)

**Example:**
```bash
curl -X PUT https://your-worker.workers.dev/template/my-template \
  -H "Authorization: Bearer YOUR_TEMPLATE_TOKEN" \
  -H "Content-Type: application/yaml" \
  --data-binary @template.yaml
```

**Response:**
- `200 OK`: Template uploaded successfully
- `400 Bad Request`: Invalid request (empty body, missing name)
- `401 Unauthorized`: Missing or invalid authorization token
- `500 Internal Server Error`: Server misconfiguration or R2 error

## Environment Variables

The worker requires the following environment variables:

- `PROFILE_TOKEN`: Token for authenticating profile conversion requests
- `TEMPLATE_TOKEN`: Token for authenticating template upload requests

## R2 Bucket Configuration

The worker uses an R2 bucket binding named `TEMPLATE` for storing templates. Configure this in `wrangler.toml`:

```toml
[[r2_buckets]]
binding = 'TEMPLATE'
bucket_name = 'template'
preview_bucket_name = 'template-dev'
```
