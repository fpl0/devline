# Use Keycloak in Devline

**Image** `quay.io/keycloak/keycloak:26.8.2` for Linux arm64. **Default host ports** `8180`, `9100`.

## Create a local realm and client

Devline supplies the running server address as `DEVLINE_KEYCLOAK_URL`. A new server has only the `master` administration realm; your application realm and client need to be configured locally. Do not use `master` as your application issuer.

Open the Admin Console at `http://127.0.0.1:8180` by default and use the bootstrap credentials shown in Devline. Create the `devline` realm, then add a client for your application's local callback URL. Use Authorization Code with PKCE for a browser or native public client. Set exact redirect URIs and web origins.

```sh
curl -fsS http://127.0.0.1:9100/health/ready
curl -fsS http://127.0.0.1:8180/realms/devline/.well-known/openid-configuration
```

Use the `issuer` value from discovery in your app. A realm URL on `localhost` and one on `127.0.0.1` are different issuer strings. Keep the browser, SDK, and token validation settings consistent. Local dev mode and its embedded database are intended for local development.

## Keep a development realm

The volume at `/opt/keycloak/data` holds the development database. For a local clone, stop Keycloak and copy the full named volume with the [cold volume backup](portable-toolbox.md). Reopen with the same version and verify the realm's discovery and login.

For a logical realm export, use `kc.sh export --realm devline --file /export/devline-realm.json` in a stopped-server maintenance container using the same image and data volume. Import into a fresh instance using `kc.sh import --file /export/devline-realm.json` with that server stopped. Realm files can include client credentials and user details. Keep them out of the repository and review the export's user options.

[Keycloak development container](https://www.keycloak.org/getting-started/getting-started-docker) · [Keycloak import and export](https://www.keycloak.org/server/importExport) · [Keycloak health](https://www.keycloak.org/observability/health)
