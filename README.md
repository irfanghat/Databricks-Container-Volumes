## Registry handshake

```bash
curl -i http://localhost:5000/v2/
```

Expected:

```http
HTTP/1.1 200 OK
Docker-Distribution-API-Version: registry/2.0
```

---

## Start an upload

```bash
curl -i \
    -X POST \
    http://localhost:5000/v2/test/blobs/uploads/
```

Expected:

```http
HTTP/1.1 202 Accepted

Location: /v2/test/blobs/uploads/<uuid>
Docker-Upload-UUID: <uuid>
Range: 0-0
Docker-Distribution-API-Version: registry/2.0
```


## Store the upload UUID

```bash
UUID=$(curl -is \
    -X POST \
    http://localhost:5000/v2/test/blobs/uploads/ \
| awk '/Docker-Upload-UUID:/ {print $2}' | tr -d '\r')

echo "$UUID"
```


## Verify the upload directory exists

If you're using the default local storage:

```bash
find ./registry/uploads
```

Or if you're using Databricks Volumes:

```bash
find /Volumes/main/images/registry/uploads
```

You should see something like:

```text
uploads/
└── 0a2d1ef2-bba5-4728-b52b-5c82b9a71f96
    └── data
```


## Stress test upload creation

Create ten upload sessions:

```bash
for i in {1..10}; do
    curl -s \
        -X POST \
        http://localhost:5000/v2/test/blobs/uploads/ \
        > /dev/null
done
```

Then count them:

```bash
find ./registry/uploads -maxdepth 1 -type d | wc -l
```

(or replace `./registry` with your configured root.)


## Verbose request

Useful while developing:

```bash
curl -v \
    -X POST \
    http://localhost:5000/v2/test/blobs/uploads/
```

This prints the full request and response headers.


### Volume Layout

```
/Volumes/workspace/default/docker_images/
├── uploads/
│   └── <uuid>/
│       └── session.json
├── blobs/
│   └── sha256/
│       ├── ab/
│       └── cd/
├── repositories/
│   └── test/
│       ├── manifests/
│       └── tags/
└── _registry/
    ├── config.json
    └── version
```