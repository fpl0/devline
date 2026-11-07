# Error codes

> Consult this page for an exhaustive list of errors you may encounter when using the Meilisearch API.

This page is an exhaustive list of Meilisearch API errors.

## `api_key_already_exists`

A key with this [`uid`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-uid) already exists.

## `api_key_not_found`

The requested API key could not be found.

## `bad_parameter`

A parameter in the request is invalid, check the error message for more information.

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `bad_request`

The request is invalid, check the error message for more information.

## `batch_not_found`

The requested batch does not exist. Please ensure that you are using the correct [`uid`](https://www.meilisearch.com/docs/reference/api/batches/list-batches).

## `chat_not_found`

No chat is configured for the specified workspace UID.

## `database_size_limit_reached`

The requested database has reached its maximum size.

## `document_not_found`

The requested document cannot be retrieved because it does not exist.

## `dump_already_processing`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `dump_not_found`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `dump_process_failed`

An error occurred during the dump creation process. The task was aborted.

## `duplicate_index_found`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `dynamic_search_rule_not_found`

This error occurs when the provided `uid` does not correspond to any configured dynamic search rule.

## `edit_documents_by_function_error`

Could not compile the provided [`function`](https://www.meilisearch.com/docs/reference/api/documents/edit-documents-by-function#body-function) as RHAI code.

## `facet_search_disabled`

The [`/facet-search`](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values) route has been queried while [the `facetSearch` index setting](https://www.meilisearch.com/docs/reference/api/settings/get-facetsearch) is set to `false`.

## `feature_not_enabled`

You have tried using an [experimental feature](https://www.meilisearch.com/docs/resources/help/experimental_features_overview) without activating it.

## `immutable_api_key_actions`

The [`actions`](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) field of an API key cannot be modified.

## `immutable_api_key_created_at`

The [`createdAt`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-created-at) field of an API key cannot be modified.

## `immutable_api_key_expires_at`

The [`expiresAt`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-expiresat) field of an API key cannot be modified.

## `immutable_api_key_indexes`

The [`indexes`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-indexes) field of an API key cannot be modified.

## `immutable_api_key_key`

The [`key`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-key) field of an API key cannot be modified.

## `immutable_api_key_uid`

The [`uid`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-uid) field of an API key cannot be modified.

## `immutable_api_key_updated_at`

The [`updatedAt`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-updated-at) field of an API key cannot be modified.

## `immutable_index_created_at`

The [`createdAt`](https://www.meilisearch.com/docs/reference/api/indexes/get-index) field of an index cannot be modified.

## `immutable_index_updated_at`

The [`updatedAt`](https://www.meilisearch.com/docs/reference/api/indexes/get-index) field of an index cannot be modified.

## `immutable_webhook`

You tried to modify a reserved [webhook](https://www.meilisearch.com/docs/reference/api/management/list-webhooks). Reserved webhooks are configured by Meilisearch Cloud and have `isEditable` set to `false`. Webhooks created with an instance option are also immutable.

## `immutable_webhook_is_editable`

You tried to manually set a webhook's `isEditable` field. Meilisearch automatically sets `isEditable` for all webhooks. Only reserved webhooks have `isEditable` set to `false`.

## `immutable_webhook_uuid`

You tried to manually set a webhook `uuid`. Meilisearch automatically generates `uuid` for webhooks.

## `import_task_already_received`

The proposed import task has already been received by this remote.

This error occurs when a cluster of Meilisearch instances are rebalancing their documents,
and an exporting instance re-exports the same documents multiple times for some reason (reboot, etc).

## `import_task_unknown_remote`

The proposed import task refers to a remote that is unknown to this Meilisearch instance.

This error occurs when a cluster of Meilisearch instances are rebalancing their documents,
and an exporting instance exports its documents using a remote name that is unexpected to this Meilisearch instance.

Check the network configuration.

## `import_task_without_network_task`

The proposed import task was received by an instance that is not in the middle of rebalancing its documents.
This error occurs when a cluster of Meilisearch instances are rebalancing their documents,
and an exporting instance exports its documents to an unsuspecting remote.

Check the network configuration.

## `inconsistent_document_change_headers`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `index_already_exists`

An index with this [`uid`](https://www.meilisearch.com/docs/reference/api/indexes/get-index) already exists, check out our guide on [index creation](https://www.meilisearch.com/docs/resources/internals/indexes).

## `index_creation_failed`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `index_not_found`

An index with this `uid` was not found, check out our guide on [index creation](https://www.meilisearch.com/docs/resources/internals/indexes).

## `index_primary_key_already_exists`

The requested index already has a primary key that [cannot be changed](https://www.meilisearch.com/docs/resources/internals/primary_key#changing-your-primary-key-with-the-update-index-endpoint).

## `index_primary_key_multiple_candidates_found`

[Primary key inference](https://www.meilisearch.com/docs/resources/internals/primary_key#meilisearch-guesses-your-primary-key) failed because the received documents contain multiple fields ending with `id`. Use the [update index endpoint](https://www.meilisearch.com/docs/reference/api/indexes/update-index) to manually set a primary key.

## `index_primary_key_no_candidate_found`

[Primary key inference](https://www.meilisearch.com/docs/resources/internals/primary_key#meilisearch-guesses-your-primary-key) failed as the received documents do not contain any fields ending with `id`. [Manually designate the primary key](https://www.meilisearch.com/docs/resources/internals/primary_key#setting-the-primary-key), or add some field ending with `id` to your documents.

## `index_scoped_api_key_with_global_action`

This error occurs when calling an endpoint that requires a global API key with an API key that is scoped to a subset of indexes.

Call the endpoint with an API key that has `"indexes": ["*"]`

## `internal`

Meilisearch experienced an internal error. Check the error message, and [open an issue](https://github.com/meilisearch/meilisearch/issues/new?assignees=\&labels=\&template=bug_report\&title=) if necessary.

## `invalid_api_key`

The requested resources are protected with an API key. The provided API key is invalid. Read more about it in our [security tutorial](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security).

## `invalid_api_key_actions`

The [`actions`](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) field for the provided API key resource is invalid. It should be an array of strings representing action names.

## `invalid_api_key_description`

The [`description`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-description) field for the provided API key resource is invalid. It should either be a string or set to `null`.

## `invalid_api_key_expires_at`

The [`expiresAt`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-expiresat) field for the provided API key resource is invalid. It should either show a future date or datetime in the [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format or be set to `null`.

## `invalid_api_key_indexes`

The [`indexes`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-indexes) field for the provided API key resource is invalid. It should be an array of strings representing index names.

## `invalid_api_key_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) parameter is invalid. It should be an integer.

## `invalid_api_key_name`

The given [`name`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-name) is invalid. It should either be a string or set to `null`.

## `invalid_api_key_offset`

The [`offset`](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) parameter is invalid. It should be an integer.

## `invalid_api_key_uid`

The given [`uid`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-uid) is invalid. The `uid` must follow the [uuid v4](https://www.sohamkamani.com/uuid-versions-explained) format.

## `invalid_batch_uids`

The [`batchUids`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) parameter is invalid. It must be an array of positive integers.

## `invalid_chat_completion_api_key`

Invalid `apiKey` parameter. It should be a string.

## `invalid_chat_completion_api_version`

Invalid API version for the chat completion. It should be a string.

## `invalid_chat_completion_base_api`

Invalid `baseUrl` parameter. It should be a valid URL to an LLM provider.

## `invalid_chat_completion_deployment_id`

Invalid `deploymentId` parameter.

## `invalid_chat_completion_org_id`

Invalid `orgId` parameter.

## `invalid_chat_completion_pre_query_prompt`

This error code is no longer emitted by Meilisearch.

## `invalid_chat_completion_project_id`

Invalid `projectId` parameter.

## `invalid_chat_completion_prompts`

This error code is no longer emitted by Meilisearch.

## `invalid_chat_completion_search_description_prompt`

Invalid `searchDescription` parameter. It should be a string.

## `invalid_chat_completion_search_filter_param_prompt`

Invalid `searchFilterParam` parameter. It should be a string.

## `invalid_chat_completion_search_index_uid_param_prompt`

Invalid `searchIndexUidParam` parameter. It should be a string.

## `invalid_chat_completion_search_query_param_prompt`

Invalid `searchQParam` parameter. It should be a string.

## `invalid_chat_completion_source`

This error code is no longer emitted by Meilisearch.

## `invalid_chat_completion_system_prompt`

Invalid system prompt. It should be a string.

## `invalid_chat_setting_document_template`

Rendering a document using the document template configured in the chat settings for this index failed. Make sure that the configured document template can render any document from the index.

## `invalid_content_type`

The [Content-Type header](https://www.meilisearch.com/docs/reference/api/headers) is not supported by Meilisearch.

* For document additions, Meilisearch supports JSON, CSV and NDJSON.
* For other routes, Meilisearch expects JSON content.

## `invalid_document_csv_delimiter`

The [`csvDelimiter`](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents) parameter is invalid. It should either be a string or [a single ASCII character](https://www.rfc-editor.org/rfc/rfc20).

## `invalid_document_edition_context`

The [`context`](https://www.meilisearch.com/docs/reference/api/documents/edit-documents-by-function#body-context) parameter is not an object.

## `invalid_document_edition_function_filter`

The [`filter`](https://www.meilisearch.com/docs/reference/api/documents/edit-documents-by-function#body-filter) parameter is not a valid Meilisearch filter.

## `invalid_document_fields`

The [`fields`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get) parameter is invalid. It should be a string.

## `invalid_document_filter`

This error occurs if:

* The [`filter`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get) parameter is invalid
  * It should be a string, array of strings, or array of array of strings for the [get documents with POST endpoint](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-post)
  * It should be a string for the [get documents with GET endpoint](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get)
* The attribute used for filtering is not defined in the [`filterableAttributes` list](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes)
* The [filter expression](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax) has a missing or invalid operator. [Read more about our supported operators](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax)

## `invalid_document_geo_field`

The provided `_geo` field of one or more documents is invalid. Meilisearch expects `_geo` to be an object with two fields, `lat` and `lng`, each containing geographic coordinates expressed as a string or floating-point number. Read more about `_geo` and how to troubleshoot it in [our dedicated guide](https://www.meilisearch.com/docs/capabilities/geo_search/getting_started).

## `invalid_document_geojson_field`

The `geojson` field in one or more documents is invalid or doesn't match the [GeoJSON specification](https://datatracker.ietf.org/doc/html/rfc7946).

## `invalid_document_id`

The provided [document identifier](https://www.meilisearch.com/docs/resources/internals/primary_key#document-id) does not meet the format requirements. A document identifier must be of type integer or string, composed only of alphanumeric characters (a-z A-Z 0-9), hyphens (-), and underscores (\_).

## `invalid_document_ids`

The [`ids`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-post#body-ids) parameter is invalid. It should be an array of valid document ids.

## `invalid_document_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get) parameter is invalid. It should be an integer.

## `invalid_document_offset`

The [`offset`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get) parameter is invalid. It should be an integer.

## `invalid_document_retrieve`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `invalid_document_retrieve_vectors`

The [`retrieveVectors`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-get#parameter-retrieve-vectors) parameter is invalid. It should be a boolean.

## `invalid_document_sort`

This error occurs if:

* The syntax for the [`sort`](https://www.meilisearch.com/docs/reference/api/documents/list-documents-with-post) parameter is invalid
* The attribute used for sorting is not defined in the [`sortableAttributes`](https://www.meilisearch.com/docs/reference/api/settings/get-sortableattributes) list or the `sort` ranking rule is missing from the settings
* A reserved keyword like `_geo`, `_geoDistance`, `_geoRadius`, or `_geoBoundingBox` is used as a filter

## `invalid_document_use_network`

The [`useNetwork`](https://www.meilisearch.com/docs/reference/api/documents/get-document#parameter-use-network) parameter is invalid. It should be a boolean.

## `invalid_dynamic_search_rule_actions`

The [`actions`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#body-actions-one-of-0) parameter is invalid.

## `invalid_dynamic_search_rule_active`

The [`active`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#body-active) parameter is invalid. It should be a boolean.

## `invalid_dynamic_search_rule_conditions`

The [`conditions`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#body-conditions-one-of-1) parameter is invalid.

## `invalid_dynamic_search_rule_description`

The [`description`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#body-description) parameter is invalid. It should be a string.

## `invalid_dynamic_search_rule_filter`

The [`filter`](https://www.meilisearch.com/docs/reference/api/search-rules/list-search-rules#body-filter-one-of-1) parameter is invalid.

## `invalid_dynamic_search_rule_filter_active`

The [`filter.active`](https://www.meilisearch.com/docs/reference/api/search-rules/list-search-rules#body-filter-one-of-1-active) parameter is invalid. It should be a boolean

## `invalid_dynamic_search_rule_filter_query`

The [`filter.query`](https://www.meilisearch.com/docs/reference/api/search-rules/list-search-rules#body-filter-one-of-1-query) parameter is invalid. It should be a string.

## `invalid_dynamic_search_rule_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/search-rules/list-search-rules#body-limit) parameter is invalid. It should be a positive integer.

## `invalid_dynamic_search_rule_offset`

The [`offset`](https://www.meilisearch.com/docs/reference/api/search-rules/list-search-rules#body-offset) parameter is invalid. It should be a positive integer.

## `invalid_dynamic_search_rule_priority`

The [`precedence`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#body-precedence) parameter is invalid. It should be a positive number

## `invalid_dynamic_search_rule_uid`

The [`uid`](https://www.meilisearch.com/docs/reference/api/search-rules/create-or-update-a-search-rule#parameter-uid) path parameter is invalid.

## `invalid_export_api_key`

The supplied security key does not have the required permissions to access the target instance.

## `invalid_export_index_filter`

The provided index export filter is not a valid [filter expression](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/advanced/filter_expression_syntax).

## `invalid_export_index_override_settings`

The [`indexes[].overrideSettings`](https://www.meilisearch.com/docs/reference/api/export/export-to-a-remote-meilisearch#body-indexes-additional-properties-override-settings) parameter is invalid. It should be a boolean.

## `invalid_export_indexes_patterns`

The provided index pattern is invalid. The index pattern must be an alphanumeric string, optionally including a wildcard.

## `invalid_export_payload_size`

The provided payload size is invalid. The payload size must be a string indicating the maximum payload size in a human-readable format.

## `invalid_export_url`

The export target instance URL is invalid or could not be reached.

If the target instance URL is in your private network, please check that it [resolves to an IP in an allowed range](https://www.meilisearch.com/docs/resources/self_hosting/sharding/manage_network#private-network-security).

## `invalid_facet_search_exhaustive_facet_count`

The [`exhaustiveFacetCount`](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values#body-exhaustive-facet-count) parameter is invalid. It should be a boolean.

## `invalid_facet_search_facet_name`

The attribute used for the `facetName` field is either not a string or not defined in the [`filterableAttributes` list](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes).

## `invalid_facet_search_name`

This error code is no longer emitted by Meilisearch.

## `invalid_facet_search_query`

The provided value for `facetQuery` is invalid. It should either be a string or `null`.

## `invalid_header_value`

A header from this request has an unexpected value.

## `invalid_index_custom_metadata`

The [`customMetadata`](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents#parameter-custom-metadata) query parameter is invalid. It should be a string.

## `invalid_index_fields_filter`

Invalid [`filter`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1) parameter.

## `invalid_index_fields_filter_attribute_patterns`

Invalid [`filter.attributePatterns`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-attribute-patterns-one-of-1) parameter. It should be an array of strings.

## `invalid_index_fields_filter_displayed`

Invalid [`filter.displayed`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-displayed) parameter. It should be a boolean.

## `invalid_index_fields_filter_distinct`

Invalid [`filter.distinct`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-distinct) parameter. It should be a boolean.

## `invalid_index_fields_filter_filterable`

Invalid [`filter.filterable`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-filterable) parameter. It should be a boolean.

## `invalid_index_fields_filter_ranking_rule`

Invalid [`filter.rankingRule`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-ranking-rule) parameter. It should be a boolean.

## `invalid_index_fields_filter_searchable`

Invalid [`filter.searchable`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-searchable) parameter. It should be a boolean.

## `invalid_index_fields_filter_sortable`

Invalid [`filter.sortable`](https://www.meilisearch.com/docs/reference/api/indexes/list-index-fields#body-filter-one-of-1-sortable) parameter. It should be a boolean.

## `invalid_index_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/indexes/list-all-indexes) parameter is invalid. It should be an integer.

## `invalid_index_offset`

The [`offset`](https://www.meilisearch.com/docs/reference/api/indexes/list-all-indexes) parameter is invalid. It should be an integer.

## `invalid_index_primary_key`

The [`primaryKey`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) field is invalid. It should either be a string or set to `null`.

## `invalid_index_uid`

There is an error in the provided index format, check out our guide on [index creation](https://www.meilisearch.com/docs/resources/internals/indexes).

## `invalid_multi_search_distinct`

This error occurs when both `federation.distinct` and the `distinct` parameter of a query are specified. Remove one of these parameters.

## `invalid_multi_search_facet_order`

Two or more indexes have a different `faceting.sortFacetValuesBy` for the same requested facet.

## `invalid_multi_search_facets`

`federation.facetsByIndex.<INDEX_NAME>` contains a value that is not in the filterable attributes list.

## `invalid_multi_search_facets_by_index`

`facetsByIndex` is not an object or contains unknown fields.

## `invalid_multi_search_federated`

The [`federation`](https://www.meilisearch.com/docs/reference/api/multi-search/perform-a-multi-search#body-federation-one-of-1) parameter is invalid. It should either be an object or set to `null`.

## `invalid_multi_search_federation_options`

This error occurs when a query inside of a federated search specifies both `useNetwork` and a non-`null` `federationOptions.remote`. Remove one of these parameters.

## `invalid_multi_search_max_values_per_facet`

`federation.mergeFacets.maxValuesPerFacet` is not a positive integer.

## `invalid_multi_search_merge_facets`

`federation.mergeFacets` is not an object or contains unexpected fields.

## `invalid_multi_search_query_facets`

A query in the queries array contains `facets` when federation is present and non-`null`.

## `invalid_multi_search_query_pagination`

A multi-search query contains `page`, `hitsPerPage`, `limit` or `offset`, but the top-level federation object is not `null`.

## `invalid_multi_search_query_personalization`

Using `.personalize` is not allowed in federated queries.

## `invalid_multi_search_query_position`

`federationOptions.queryPosition` is not a positive integer.

## `invalid_multi_search_query_ranking_rules`

Two or more queries in a multi-search request have incompatible results.

## `invalid_multi_search_query_show_performance_details`

[`showPerformanceDetails`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-performance-details) was used at the search query level in a federated search. Use [`federation.showPerformanceDetails`](https://www.meilisearch.com/docs/reference/api/multi-search/perform-a-multi-search#body-federation-one-of-1-show-performance-details) in federated search requests.

## `invalid_multi_search_remote`

`federationOptions.remote` is not `network.self` and is not a key in `network.remotes`.

## `invalid_multi_search_weight`

A multi-search query contains a negative value for `federated.weight`.

## `invalid_network_leader`

The error occurs if:

* The `leader` parameter is not a string.
* The specified leader is not part of the declared remotes.
* The specified leader is not `self`.

## `invalid_network_remotes`

The [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) contains a `remotes` that is not an object or `null`.

## `invalid_network_search_api_key`

One of the remotes in the [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) contains a `searchApiKey` that is not a string or `null`.

## `invalid_network_self`

The [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) contains a `self` that is not a string or `null`.

## `invalid_network_shards`

This error occurs if:

* The [`shards`](https://www.meilisearch.com/docs/reference/api/experimental-features/configure-network-topology#body-shards) parameter is invalid. It must be an object or `null`.
* The `shards` parameter contains at least one shard referencing an undeclared remote.
* The `shards` parameter contains at least one shard referencing no remotes.

## `invalid_network_url`

One of the remotes in the [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) contains a `url` that is not a string.

## `invalid_network_write_api_key`

One of the remotes in the [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) contains a `writeApiKey` that is not a string or `null`.

## `invalid_render_input`

The [`input`](https://www.meilisearch.com/docs/reference/api/template/render-template#body-input-one-of-1) parameter is invalid.

## `invalid_render_template`

The [`template`](https://www.meilisearch.com/docs/reference/api/template/render-template#body-template) parameter is invalid.

## `invalid_s_3_snapshot_parameters`

Invalid [S3 snapshot configuration](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#s3-options-enterprise)

## `invalid_s_3_snapshot_request`

This Meilisearch instance sent a bad S3 snapshot request.

## `invalid_search_attributes_to_crop`

The [`attributesToCrop`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-crop) parameter is invalid. It should be an array of strings, a string, or set to `null`.

## `invalid_search_attributes_to_highlight`

The [`attributesToHighlight`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-highlight) parameter is invalid. It should be an array of strings, a string, or set to `null`.

## `invalid_search_attributes_to_retrieve`

The [`attributesToRetrieve`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-retrieve) parameter is invalid. It should be an array of strings, a string, or set to `null`.

## `invalid_search_attributes_to_search_on`

The value passed to [`attributesToSearchOn`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-search-on) is invalid. `attributesToSearchOn` accepts an array of strings indicating document attributes. Attributes given to `attributesToSearchOn` must be present in the [`searchableAttributes` list](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_searchable_attributes#update-searchable-attributes).

## `invalid_search_crop_length`

The [`cropLength`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-crop-length) parameter is invalid. It should be an integer.

## `invalid_search_crop_marker`

The [`cropMarker`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-crop-marker) parameter is invalid. It should be a string or set to `null`.

## `invalid_search_distinct`

This error occurs if:

* [`distinct`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-distinct) is not a string.
* The selected distinct attribute is not a filterable attribute.

## `invalid_search_embedder`

[`embedder`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-hybrid) is invalid. It should be a string corresponding to the name of a configured embedder.

## `invalid_search_facets`

This error occurs if:

* The [`facets`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-facets) parameter is invalid. It should be an array of strings, a string, or set to `null`
* The attribute used for faceting is not defined in the [`filterableAttributes` list](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes)

## `invalid_search_filter`

This error occurs if:

* The syntax for the [`filter`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-filter) parameter is invalid
* The attribute used for filtering is not defined in the [`filterableAttributes` list](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes)
* A reserved keyword like `_geo`, `_geoDistance`, or `_geoPoint` is used as a filter

## `invalid_search_highlight_post_tag`

The [`highlightPostTag`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-highlight-pre-tag) parameter is invalid. It should be a string.

## `invalid_search_highlight_pre_tag`

The [`highlightPreTag`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-highlight-pre-tag) parameter is invalid. It should be a string.

## `invalid_search_hits_per_page`

The [`hitsPerPage`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-hits-per-page) parameter is invalid. It should be an integer.

## `invalid_search_hybrid_query`

The [`hybrid`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-hybrid) parameter is neither `null` nor an object, or it is an object with unknown keys.

## `invalid_search_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-limit) parameter is invalid. It should be an integer.

## `invalid_search_locales`

The [`locales`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-locales) parameter is invalid.

## `invalid_search_matching_strategy`

The [`matchingStrategy`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-matching-strategy) parameter is invalid. It should either be set to `last` or `all`.

## `invalid_search_media`

The value passed to [`media`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-media) is not a valid JSON object.

## `invalid_search_media_and_vector`

The search query contains non-`null` values for both [`media`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-media) and [`vector`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-vector). These two parameters are mutually exclusive, since `media` generates vector embeddings via the embedder configured in `hybrid`.

## `invalid_search_offset`

The [`offset`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-offset) parameter is invalid. It should be an integer.

## `invalid_search_page`

The [`page`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-page) parameter is invalid. It should be an integer.

## `invalid_search_personalize`

[`personalize`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-personalize-one-of-1) is not an object with the expected keys.

## `invalid_search_personalize_user_context`

The [`personalizeUserContext`](https://www.meilisearch.com/docs/reference/api/search/search-with-get#parameter-personalize-user-context) query parameter is invalid. It should be a string

## `invalid_search_q`

The [`q`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-q) parameter is invalid. It should be a string or set to `null`

## `invalid_search_ranking_score_threshold`

The [`rankingScoreThreshold`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-ranking-score-threshold) in a search or multi-search request is not a number between `0.0` and `1.0`.

## `invalid_search_retrieve_vectors`

The [`retrieveVectors`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-retrieve-vectors) parameter is invalid. It should be a boolean.

## `invalid_search_semantic_ratio`

The [`hybrid.semanticRatio`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-hybrid-one-of-1-semantic-ratio) parameter is invalid. It should be a number between 0.0 and 1.0

## `invalid_search_show_matches_position`

The [`showMatchesPosition`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-matches-position) parameter is invalid. It should either be a boolean or set to `null`.

## `invalid_search_show_performance_details`

[`showPerformanceDetails`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-performance-details) is invalid. It should be a boolean.

## `invalid_search_show_ranking_score`

[`showRankingScore`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-ranking-score) is invalid. It should be a boolean.

## `invalid_search_show_ranking_score_details`

[`showRankingScoreDetails`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-ranking-score-details) is invalid. It should be a boolean.

## `invalid_search_sort`

This error occurs if:

* The syntax for the [`sort`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-sort) parameter is invalid
* The attribute used for sorting is not defined in the [`sortableAttributes`](https://www.meilisearch.com/docs/reference/api/settings/get-sortableattributes) list or the `sort` ranking rule is missing from the settings
* A reserved keyword like `_geo`, `_geoDistance`, `_geoRadius`, or `_geoBoundingBox` is used as a filter

## `invalid_search_use_network`

[`useNetwork`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-use-network) is invalid. It should be a boolean.

## `invalid_search_vector`

The [`vector`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-vector) parameter is invalid. It should be an array of numbers.

## `invalid_settings_dictionary`

The value of the [dictionary](https://www.meilisearch.com/docs/reference/api/settings/update-dictionary) is invalid.

## `invalid_settings_displayed_attributes`

The value of [displayed attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_displayed_attributes#set-displayed-attributes) is invalid. It should be an empty array, an array of strings, or set to `null`.

## `invalid_settings_distinct_attribute`

The value of [distinct attributes](https://www.meilisearch.com/docs/capabilities/full_text_search/how_to/configure_distinct_attribute) is invalid. It should be a string or set to `null`.

## `invalid_settings_embedders`

The [`embedders`](https://www.meilisearch.com/docs/reference/api/settings/get-embedders) index setting value is invalid.

## `invalid_settings_facet_search`

The [`facetSearch`](https://www.meilisearch.com/docs/reference/api/settings/get-facetsearch) index setting value is invalid.

## `invalid_settings_faceting`

The [`faceting`](https://www.meilisearch.com/docs/reference/api/settings/update-faceting) index setting is invalid.

## `invalid_settings_filterable_attributes`

The value of [filterable attributes](https://www.meilisearch.com/docs/reference/api/settings/get-filterableattributes) is invalid. It should be an empty array, an array of strings, or set to `null`.

## `invalid_settings_foreign_keys`

The value of [foreign keys](https://www.meilisearch.com/docs/reference/api/settings/update-foreignkeys) is invalid.

## `invalid_settings_index_chat`

The [chat](https://www.meilisearch.com/docs/reference/api/settings/update-chat) index settings are invalid.

## `invalid_settings_localized_attributes`

The [`localizedAttributes`](https://www.meilisearch.com/docs/reference/api/settings/get-localizedattributes) index setting value is invalid.

## `invalid_settings_non_separator_tokens`

The value of [non-separator tokens](https://www.meilisearch.com/docs/reference/api/settings/update-nonseparatortokens) is invalid.

## `invalid_settings_pagination`

The value for the [`maxTotalHits`](https://www.meilisearch.com/docs/reference/api/settings/update-pagination) field is invalid. It should either be an integer or set to `null`.

## `invalid_settings_prefix_search`

The [`prefixSearch`](https://www.meilisearch.com/docs/reference/api/settings/get-prefixsearch) index setting value is invalid.

## `invalid_settings_proximity_precision`

[`proximityPrecision`](https://www.meilisearch.com/docs/reference/api/settings/update-proximityprecision#update-proximityprecision) does not have one of the allowed values.

## `invalid_settings_ranking_rules`

This error occurs if:

* The [settings payload](https://www.meilisearch.com/docs/reference/api/settings/update-all-settings) has an invalid format
* A non-existent ranking rule is specified
* A custom ranking rule is malformed
* A reserved keyword like `_geo`, `_geoDistance`, `_geoRadius`, `_geoBoundingBox`, or `_geoPoint` is used as a custom ranking rule

## `invalid_settings_search_cutoff_ms`

The specified value for [`searchCutoffMs`](https://www.meilisearch.com/docs/reference/api/settings/update-searchcutoffms) is invalid. It should be an integer indicating the cutoff in milliseconds.

## `invalid_settings_searchable_attributes`

The value of [searchable attributes](https://www.meilisearch.com/docs/reference/api/settings/get-searchableattributes) is invalid. It should be an empty array, an array of strings or set to `null`.

## `invalid_settings_separator_tokens`

The value of [separator tokens](https://www.meilisearch.com/docs/reference/api/settings/update-separatortokens) is invalid.

## `invalid_settings_sortable_attributes`

The value of [sortable attributes](https://www.meilisearch.com/docs/reference/api/settings/get-sortableattributes) is invalid. It should be an empty array, an array of strings or set to `null`.

## `invalid_settings_stop_words`

The value of [stop words](https://www.meilisearch.com/docs/reference/api/settings/get-stopwords) is invalid. It should be an empty array, an array of strings or set to `null`.

## `invalid_settings_synonyms`

The value of the [synonyms](https://www.meilisearch.com/docs/reference/api/settings/get-synonyms) is invalid. It should either be an object or set to `null`.

## `invalid_settings_typo_tolerance`

This error occurs if:

* The [`enabled`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) field is invalid. It should either be a boolean or set to `null`
* The [`disableOnAttributes`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) field is invalid. It should either be an array of strings or set to `null`
* The [`disableOnWords`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) field is invalid. It should either be an array of strings or set to `null`
* The [`minWordSizeForTypos`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) field is invalid. It should either be an integer or set to `null`
* The value of either [`oneTypo`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) or [`twoTypos`](https://www.meilisearch.com/docs/reference/api/settings/get-typotolerance) is invalid. It should either be an integer or set to `null`

## `invalid_similar_attributes_to_retrieve`

[`attributesToRetrieve`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-attributes-to-retrieve) is invalid. It should be an array of strings, a string, or set to null.

## `invalid_similar_embedder`

[`embedder`](https://www.meilisearch.com/docs/reference/api/similar-documents/get-similar-documents-with-post) is invalid. It should be a string corresponding to the name of a configured embedder.

## `invalid_similar_filter`

[`filter`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-filter) is invalid or contains a filter expression with a missing or invalid operator. Filter expressions must be a string, array of strings, or array of array of strings for the POST endpoint. It must be a string for the GET endpoint.

Meilisearch also throws this error if the attribute used for filtering is not defined in the `filterableAttributes` list.

## `invalid_similar_id`

The provided target document identifier is invalid. A document identifier can be of type integer or string, only composed of alphanumeric characters (a-z A-Z 0-9), hyphens (-) and underscores (\_).

## `invalid_similar_limit`

[`limit`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-limit) is invalid. It should be an integer.

## `invalid_similar_offset`

[`offset`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-offset) is invalid. It should be an integer.

## `invalid_similar_ranking_score_threshold`

The [`rankingScoreThreshold`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-ranking-score-threshold) in a similar documents request is not a number between `0.0` and `1.0`.

## `invalid_similar_retrieve_vectors`

The [`retrieveVectors`](https://www.meilisearch.com/docs/reference/api/similar-documents/get-similar-documents-with-post#body-retrieve-vectors) parameter is invalid. It should be a boolean.

## `invalid_similar_show_performance_details`

[`showPerformanceDetails`](https://www.meilisearch.com/docs/reference/api/similar-documents/get-similar-documents-with-get#parameter-show-performance-details) is invalid. It should be a boolean.

## `invalid_similar_show_ranking_score`

[`showRankingScore`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-ranking-score) is invalid. It should be a boolean.

## `invalid_similar_show_ranking_score_details`

[`ranking_score_details`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-show-ranking-score-details) is invalid. It should be a boolean.

## `invalid_skip_creation`

The [`skipCreation`](https://www.meilisearch.com/docs/reference/api/documents/add-or-replace-documents#parameter-skip-creation) query parameter is invalid. It should be a boolean.

## `invalid_state`

The database is in an invalid state. Deleting the database and re-indexing should solve the problem.

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `invalid_stats_show_internal_database_sizes`

The [`showInternalDatabaseSizes`](https://www.meilisearch.com/docs/reference/api/indexes/get-stats-of-index#parameter-show-internal-database-sizes) query parameter is invalid. It should be a boolean.

## `invalid_stats_size_format`

The [`sizeFormat`](https://www.meilisearch.com/docs/reference/api/indexes/get-stats-of-index#parameter-size-format) query parameter is invalid.

## `invalid_store_file`

The `data.ms` folder is in an invalid state. Your `b` file is corrupted or the `data.ms` folder has been replaced by a file.

## `invalid_swap_duplicate_index_found`

The indexes used in the [`indexes`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) array for a [swap index](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) request have been declared multiple times. You must declare each index only once.

## `invalid_swap_indexes`

This error happens if:

* The payload doesn't contain exactly two index [`uids`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) for a swap operation
* The payload contains an invalid index name in the [`indexes`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) array

## `invalid_swap_rename`

The [`rename`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes#body-items-rename) parameter is invalid. It should be a boolean.

## `invalid_task_after_enqueued_at`

The [`afterEnqueuedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_after_finished_at`

The [`afterFinishedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_after_started_at`

The [`afterStartedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_before_enqueued_at`

The [`beforeEnqueuedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_before_finished_at`

The [`beforeFinishedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_before_started_at`

The [`beforeStartedAt`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_task_canceled_by`

The [`canceledBy`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid. It should be an integer. Multiple `uid`s should be separated by commas (`,`).

## `invalid_task_from`

The [`from`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) parameter is invalid. It must be a positive integer.

## `invalid_task_limit`

The [`limit`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) parameter is invalid. It must be a positive integer.

## `invalid_task_reverse`

The [`reverse`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) parameter is invalid. It must be a boolean.

## `invalid_task_statuses`

The requested task status is invalid. Please use one of the [possible values](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

## `invalid_task_types`

The requested task type is invalid. Please use one of the [possible values](https://www.meilisearch.com/docs/reference/api/tasks/get-task).

## `invalid_task_uids`

The [`uids`](https://www.meilisearch.com/docs/reference/api/tasks/list-tasks) query parameter is invalid.

## `invalid_vector_dimensions`

This error occurs:

* When an embedder returns a vector whose dimensions are not consistent with the declared dimensions for this embedder. Check the declared dimensions for this embedder.
* When a document contains a manually provided vector whose dimensions are not consistent with the declared dimensions for this embedder. Check the declared dimensions for this embedder and the vector associated to the document.
* Internal: when attempting to insert a vector whose dimensions are not consistent with the dimensions of vectors in the vector store. Check the error message, and [open an issue](https://github.com/meilisearch/meilisearch/issues/new?assignees=\&labels=\&template=bug_report\&title=) if necessary.

## `invalid_vectors_type`

The [`_vectors`](https://www.meilisearch.com/docs/capabilities/hybrid_search/how_to/search_with_user_provided_embeddings#add-documents-to-meilisearch) field of a document is invalid

## `invalid_webhook_headers`

The provided webhook `headers` field is not a JSON object or not a valid HTTP header. Meilisearch also returns this error if you set more than 200 header fields for a single webhook.

## `invalid_webhook_url`

The provided webhook URL isn’t a valid JSON string, is `null`, is missing, or its value cannot be parsed as a valid URL.

## `invalid_webhook_uuid`

The provided webhook `uuid` is not a valid UUID.

## `invalid_webhooks`

The create webhook request did not contain a valid JSON payload. Meilisearch also returns this error when you try to create more than 20 webhooks.

## `io_error`

This error generally occurs when the host system has no space left on the device or when the database doesn't have read or write access.

## `malformed_payload`

The [Content-Type header](https://www.meilisearch.com/docs/reference/api/headers) does not match the request body payload format or the format is invalid.

## `max_fields_limit_exceeded`

The index exceeds the [maximum limit of 65,536 attributes](https://www.meilisearch.com/docs/resources/help/known_limitations#maximum-number-of-attributes-per-index).

## `missing_api_key_actions`

The [`actions`](https://www.meilisearch.com/docs/reference/api/keys/list-api-keys) field is missing from payload.

## `missing_api_key_expires_at`

The [`expiresAt`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-expiresat) field is missing from payload.

## `missing_api_key_indexes`

The [`indexes`](https://www.meilisearch.com/docs/reference/api/keys/get-api-key#response-indexes) field is missing from payload.

## `missing_authorization_header`

This error happens if:

* The requested resources are protected with an API key that was not provided in the request header. Check our [security tutorial](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security) for more information

## `missing_content_type`

The payload does not contain a [Content-Type header](https://www.meilisearch.com/docs/reference/api/headers).

* For document additions, Meilisearch supports JSON, CSV and NDJSON.
* For other routes, Meilisearch expects JSON content.

## `missing_document_edition_function`

The [`function`](https://www.meilisearch.com/docs/reference/api/documents/edit-documents-by-function#body-function) parameter is missing.

## `missing_document_filter`

The [`filter`](https://www.meilisearch.com/docs/reference/api/documents/delete-documents-by-filter#body-filter) parameter is missing.

## `missing_document_id`

A document does not contain any value for the required primary key, and is thus invalid. Check documents in the current addition for the invalid ones.

## `missing_facet_search_facet_name`

The [`facetName`](https://www.meilisearch.com/docs/reference/api/facet-search/search-for-facet-values) parameter is required.

## `missing_index_uid`

The payload is missing the [`uid`](https://www.meilisearch.com/docs/reference/api/indexes/get-index) field.

## `missing_master_key`

You need to set a master key before you can access the `/keys` route. Read more about setting a master key at launch in our [security tutorial](https://www.meilisearch.com/docs/resources/self_hosting/security/basic_security).

## `missing_network_url`

One of the remotes in the [network object](https://www.meilisearch.com/docs/reference/api/network/get-network) does not contain the `url` field.

## `missing_payload`

The Content-Type header was specified, but no request body was sent to the server or the request body is empty.

## `missing_search_hybrid`

The [`hybrid`](https://www.meilisearch.com/docs/reference/api/search/search-with-post#body-hybrid-one-of-1) parameter is missing when `vector` or `media` were provided.

## `missing_swap_indexes`

The index swap payload is missing the [`indexes`](https://www.meilisearch.com/docs/reference/api/indexes/swap-indexes) object.

## `missing_task_filters`

The [cancel tasks](https://www.meilisearch.com/docs/reference/api/tasks/cancel-tasks) and [delete tasks](https://www.meilisearch.com/docs/reference/api/tasks/delete-tasks) endpoints require one of the available query parameters.

## `network_version_mismatch`

The network version of the current task does not match the current version.

Check that the sender belongs to the current network. Check the network configuration.

## `network_version_too_old`

The task could not be processed because its version was older than the current network version.

## `no_space_left_on_device`

This error occurs if:

* The host system partition reaches its maximum capacity and can no longer accept writes
* The tasks queue reaches its limit and can no longer accept writes. You can delete tasks using the [delete tasks endpoint](https://www.meilisearch.com/docs/reference/api/tasks/delete-tasks) to continue write operations
* While indexing, the temporary storage partition (for example, `/tmp`) ran out and can no longer accept writes

## `not_found_similar_id`

Meilisearch could not find the target document. Make sure your target document identifier corresponds to a document in your index.

## `not_leader`

The task was sent to a remote which is not the leader of the network. Send the task to the leader of the network.

## `payload_too_large`

The payload sent to the server was too large. Check out this [guide](https://www.meilisearch.com/docs/resources/self_hosting/configuration/reference#payload-limit-size) to customize the maximum payload size accepted by Meilisearch.

## `receive_import_finished_unknown_remote`

The proposed import finish signal refers to a remote that is unknown to this Meilisearch instance.

This error occurs when a cluster of Meilisearch instances are rebalancing their documents,
and an exporting instance exports its documents using a remote name that is unexpected to this Meilisearch instance.

Check the network configuration.

## `remote_bad_request`

The remote instance answered with `400 BAD REQUEST`.

## `remote_bad_response`

The remote instance answered with a response that this instance could not use as a federated search response.

## `remote_could_not_send_request`

There was an error while sending the remote federated search request.

## `remote_invalid_api_key`

The remote instance answered with `403 FORBIDDEN` or `401 UNAUTHORIZED` to this instance’s request. The configured API keys are either missing, invalid, or lack the required permissions.

## `remote_remote_error`

The remote instance answered with `500 INTERNAL ERROR`.

## `remote_timeout`

The remote did not answer in the allocated time.

## `render_document_not_found`

The requested document was not found.

## `requires_enterprise_edition`

Using this feature requires the [Enterprise Edition](https://www.meilisearch.com/docs/resources/self_hosting/enterprise_edition) of Meilisearch.

## `s_3_snapshot_server_error`

The S3 server returned an HTTP 5xx error.

## `task_file_not_found`

The task requires a payload stored on disk, but it could not be opened.

## `task_not_found`

The requested task uid does not exist. Please ensure that you are using the correct `uid`.

## `template_parsing_error`

Could not parse the specified [document template](https://www.meilisearch.com/docs/getting_started/glossary#document-template)

## `template_rendering_error`

Could not render the specified [document template](https://www.meilisearch.com/docs/getting_started/glossary#document-template)

## `too_many_open_files`

Reached the maximum number of simultaneously opened files. Raise the OS' limit of open files.

## `too_many_search_requests`

This Meilisearch instance is currently experiencing a high volume of search requests. Retry later.

## `too_many_vectors`

A given document would have more than the accepted limit of embeddings for an embedder.

## `unexpected_network_previous_remotes`

The reserved field `previousRemotes` was used. Remove this field.

## `unimplemented_external_function_calling`

Custom tools are not yet supported in chat completion requests. Remove custom tools from the `tools` parameter.

## `unimplemented_multi_choice_chat_completions`

Unsupported value of the `n` parameter. The only supported value is `1`.

## `unimplemented_non_streaming_chat_completions`

Non-streaming chat completion is not implemented. Use the streaming variant.

## `unknown_remote`

Attempt to contact a remote that was not declared in the network configuration. Check the network configuration and the remote name in the request.

## `unprocessed_network_task`

One or multiple remotes of the network are already processing a network task. Cancel these tasks or let them finish processing before sending another network task.

## `unretrievable_document`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `unretrievable_error_code`

This error code is no longer emitted and can be found in tasks imported from a dump created using a previous version of Meilisearch.

## `unsupported_media_type`

The media type of the request is not supported.

## `vector_embedding_error`

The batch failed because one or more documents could not be embedded. Read the error message for specifics.

## `webhook_not_found`

The provided webhook `uuid` does not correspond to any configured webhooks in the instance.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.