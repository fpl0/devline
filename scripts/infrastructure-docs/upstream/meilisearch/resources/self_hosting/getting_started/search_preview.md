# Search preview

> Meilisearch comes with a built-in search interface for quick testing during development.

Meilisearch Cloud gives you access to a dedicated search preview interface. This is useful to test search result relevancy when you are tweaking an index's settings.

> [!NOTE]
>
> If you are self-hosting Meilisearch and need a local search interface, access `http://localhost:7700` in your browser. This local preview only allows you to perform plain searches and offers no customization options.

## Accessing and using search preview

Log into your [Meilisearch Cloud](https://cloud.meilisearch.com/login) account, navigate to your project, then click on "Search preview":

![Meilisearch Cloud's project menu with the last option, 'Search preview', selected](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/01-search-preview-menu.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=f76987596aa2549e5b7a3bdcf01eabe3)

Select the index you want to search on using the input on the left-hand side:

![Meilisearch Cloud's search preview interface, with the index selecting input highlighted](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/02-select-index.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=77ea975607d145e3a40bf7213a03d47f)

Then use the main input to perform plain keyword searches:

![Meilisearch Cloud's search preview interface, with the search input selected and containing a search string](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/03-basic-search.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=c75c91a2bc553a7866088ba7239eeb61)

When debugging relevancy, you may want to activate the "Ranking score" option. This displays the overall [ranking score](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/ranking_score) for each result, together with the score for each individual ranking rule:

![The same search preview interface as in the previous image, but with the 'Ranking score' option turned on. Search results are the same, but include the document's ranking score](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/04-score.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=fcf5955fde56ef32db913e2e49d87098)

## Configuring search options

Use the menu on the left-hand side to configure [sorting](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/how_to/sort_results) and [filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started). These require you to first edit your index's sortable and filterable attributes. You may additionally configure any filterable attributes as facets. In this example, "Genres" is one of the configured facets:

![The sidebar of the search preview interface, with a handful of options, including 'Sort by', 'AI-powered search', 'Filters', and 'Genres'](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/05-sidebar-options.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=496df2a204905ce1790e751f27c85d95)

You can also perform [AI-powered searches](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started) if this functionality has been enabled for your project.

Clicking on "Advanced parameters" gives you access to further customization options, including setting which document fields Meilisearch returns and explicitly declaring the search language:

![The same sidebar as before with the 'Advanced parameters' option highlighted](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/06-sidebar-options-advanced.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=18df4070ee16ef721ffd1ce0eade99da)

## Exporting search options

You can export the full search query for further testing in other tools and environments. Click on the cloud icon next to "Advanced parameters", then choose to download a JSON file or copy the query to your clipboard:

![The same sidebar as before with the 'Export' option highlighted](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/search_preview/07-sidebar-options-export.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=9e6674fb128f25eaaa250fdf3d8b96f1)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.