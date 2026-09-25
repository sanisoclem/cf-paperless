# cf-paperless

Cloudflare Email Worker that stores attachments to R2.

## Requirements

- `worker-build`: `cargo install worker-build`
- [fnm](https://github.com/Schniz/fnm) and [direnv](https://direnv.net) (optional)

```sh
direnv allow
npm install
```

## Setup

1. Create R2 Bucket named `paperless-inbox`
1. Create CF API Key for cluster. It needs `Object Read & Write` to the bucket
1. Enable email routing on the domain (must be in Cloudflare)
1. Create worker project in Cloudflare, don't forget to set `ALLOWED_SENDERS` (e.g. `me@proton.me,*@example.com`) and `ALLOWED_EXTENSIONS` (e.g. `pdf,png,jpg`)
1. Route email to worker
