# Postman collection for Meilisearch

> Import Meilisearch's OpenAPI specification into Postman to test and debug the API with a ready-made collection.

Postman is a platform that lets you create, organize, and reuse HTTP requests. You can import the Meilisearch OpenAPI specification directly into Postman to get a complete, up-to-date collection of all API routes.

If you don't have Postman already, you can [download it here](https://www.postman.com/downloads/). It's free and available on many OS distributions.

## Prerequisites

Download the Meilisearch OpenAPI specification file from the [latest Meilisearch release on GitHub](https://github.com/meilisearch/meilisearch/releases/latest). Scroll down to the **Assets** section of the release and download `meilisearch-openapi.json`. Save it somewhere on your computer.

## Import the OpenAPI specification

Click the three-dot menu (**...**) at the top of the sidebar, then select **Import**.

![The three-dot menu in the Postman sidebar with the 'Import' option highlighted](https://mintcdn.com/meilisearch-6b28dec2/OfKxpiaME4eTUIOs/assets/images/postman/import_menu.png?fit=max\&auto=format\&n=OfKxpiaME4eTUIOs\&q=85\&s=fe08aa4ad01fe8056137f9daf7628634)

In the import dialog, drag and drop the OpenAPI specification file or click **files** to select it from your computer.

![The Postman import dialog with a drop zone for files](https://mintcdn.com/meilisearch-6b28dec2/OfKxpiaME4eTUIOs/assets/images/postman/import_dialog.png?fit=max\&auto=format\&n=OfKxpiaME4eTUIOs\&q=85\&s=20c0e9be5e329490be969bd416a1e6c3)

Postman detects the file format and asks how to import it. Select **Postman Collection** to create a collection with all Meilisearch API routes, then click **Import**.

![The import specification dialog with 'Postman Collection' selected](https://mintcdn.com/meilisearch-6b28dec2/OfKxpiaME4eTUIOs/assets/images/postman/import_postman_collection.png?fit=max\&auto=format\&n=OfKxpiaME4eTUIOs\&q=85\&s=ed2bcb9db07c30ac8518fa33055a6889)

> [!NOTE]
>
> You can also select **OpenAPI 3.1 Specification with a Postman Collection** if you want to keep the raw specification alongside the collection.
>
> ![The import specification dialog with 'OpenAPI 3.1 Specification with a Postman Collection' selected](https://mintcdn.com/meilisearch-6b28dec2/OfKxpiaME4eTUIOs/assets/images/postman/import_openapi.png?fit=max\&auto=format\&n=OfKxpiaME4eTUIOs\&q=85\&s=3aa9235abaaa0f22342cb950479c1d79)

## Configure authentication

After importing, you need to configure your API key so Postman can authenticate requests to your Meilisearch instance.

Individual requests use **Bearer Token** authentication. You can verify this by selecting any request and checking the **Authorization** tab. The token field references `{{bearerToken}}`, which Postman resolves from your global variables.

![The Postman collection showing all Meilisearch API routes with the Authorization tab open and Bearer Token configured](https://mintcdn.com/meilisearch-6b28dec2/OfKxpiaME4eTUIOs/assets/images/postman/collection_overview.png?fit=max\&auto=format\&n=OfKxpiaME4eTUIOs\&q=85\&s=6708fb453db7944f5ae080e59b2856b4)

## Start using the collection

You can now select any route from the sidebar, adjust parameters as needed, and click **Send** to make requests to your Meilisearch instance.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.