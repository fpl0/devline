# Use SeaweedFS S3 in Devline

**Image** `docker.io/chrislusf/seaweedfs:4.52` for Linux arm64. **Default host ports** `8333`.

## Connect to S3

This single-node mini deployment creates a `devline` bucket. Use the S3 endpoint and generated secret from Devline. Configure the SDK with access key `devline`, region `us-east-1`, and path-style addressing. The service endpoint is normally `http://127.0.0.1:8333`.

```sh
curl -fsS http://127.0.0.1:8333/healthz
printf 'offline object\n' > devline-probe.txt
curl -fsS --aws-sigv4 'aws:amz:us-east-1:s3' \
  --user "devline:$AWS_SECRET_ACCESS_KEY" --upload-file devline-probe.txt \
  http://127.0.0.1:8333/devline/devline-probe.txt
curl -fsS --aws-sigv4 'aws:amz:us-east-1:s3' \
  --user "devline:$AWS_SECRET_ACCESS_KEY" \
  http://127.0.0.1:8333/devline/devline-probe.txt
```

Use the generated secret in a local environment variable. If an SDK builds `devline.127.0.0.1` instead of adding `/devline` to the endpoint path, set its force-path-style option.

## Back up objects and metadata

A simple logical copy uses a prepared S3 client to download the bucket objects. This captures object contents. Use a cold copy of `/data` when you also need the local filer metadata, volume files, and mini configuration. Stop the service before copying. Restore the complete directory into a fresh volume and verify a signed request and a known object's contents.

[SeaweedFS S3 reference](https://github.com/seaweedfs/seaweedfs/wiki/Amazon-S3-API) · [SeaweedFS HTTP API reference](https://github.com/seaweedfs/seaweedfs/blob/4.52/REST_API.md)
