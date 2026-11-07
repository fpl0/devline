# Symfony

> Integrate Meilisearch with Symfony using the official bundle.

The official [meilisearch-symfony](https://github.com/meilisearch/meilisearch-symfony) bundle provides seamless integration between Meilisearch and Symfony applications with Doctrine ORM support.

## Prerequisites

* PHP 7.4 or higher
* Symfony 5.4 or higher
* Doctrine ORM (optional, for automatic entity indexing)
* A Meilisearch instance ([Cloud](https://cloud.meilisearch.com) or [self-hosted](https://www.meilisearch.com/docs/resources/self_hosting/getting_started/quick_start))

## 1. Install the bundle

```bash
composer require meilisearch/search-bundle
```

## 2. Configure the bundle

Create or update `config/packages/meilisearch.yaml`:

```yaml
meilisearch:
  url: '%env(MEILISEARCH_URL)%'
  api_key: '%env(MEILISEARCH_KEY)%'
```

Add to your `.env` file:

```bash
MEILISEARCH_URL=https://your-instance.meilisearch.io
MEILISEARCH_KEY=your_api_key
```

> [!NOTE]
>
> [Get a free Cloud instance →](https://cloud.meilisearch.com)

## 3. Configure an entity for indexing

Register your entity in `config/packages/meilisearch.yaml`:

```yaml
meilisearch:
  url: '%env(MEILISEARCH_URL)%'
  api_key: '%env(MEILISEARCH_KEY)%'
  indices:
    - name: movies
      class: App\Entity\Movie
```

In your entity, implement `getSearchableArray()` to control which fields are indexed:

```php
<?php

namespace App\Entity;

use Doctrine\ORM\Mapping as ORM;

#[ORM\Entity]
class Movie
{
    #[ORM\Id]
    #[ORM\GeneratedValue]
    #[ORM\Column]
    private ?int $id = null;

    #[ORM\Column(length: 255)]
    private ?string $title = null;

    #[ORM\Column]
    private ?int $year = null;

    #[ORM\Column(type: 'json')]
    private array $genres = [];

    // Getters and setters...

    public function getSearchableArray(): array
    {
        return [
            'id' => $this->id,
            'title' => $this->title,
            'year' => $this->year,
            'genres' => $this->genres,
        ];
    }
}
```

## 4. Index your data

Import existing entities to Meilisearch:

```bash
php bin/console meilisearch:import
```

New entities are automatically indexed when created or updated via Doctrine.

## 5. Search

Use `SearchManagerInterface` to search your indexed entities:

```php
<?php

namespace App\Controller;

use Meilisearch\Bundle\SearchManagerInterface;
use App\Entity\Movie;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Annotation\Route;

class SearchController extends AbstractController
{
    #[Route('/search', name: 'search')]
    public function search(SearchManagerInterface $searchManager): Response
    {
        $results = $searchManager->search(
            Movie::class,
            'matrix'
        );

        return $this->render('search/results.html.twig', [
            'movies' => $results,
        ]);
    }
}
```

## 6. Search with filters

Add index settings to your `config/packages/meilisearch.yaml`:

```yaml
meilisearch:
  url: '%env(MEILISEARCH_URL)%'
  api_key: '%env(MEILISEARCH_KEY)%'
  indices:
    - name: movies
      class: App\Entity\Movie
      settings:
        filterableAttributes: ['genres', 'year']
        sortableAttributes: ['year']
```

Update the index settings:

```bash
php bin/console meilisearch:create
```

Then search with filters:

```php
$results = $searchManager->search(
    Movie::class,
    'action',
    [
        'filter' => 'year > 2000',
        'sort' => ['year:desc'],
    ]
);
```

## Available commands

| Command | Description |
| - | - |
| `meilisearch:import` | Import all entities to Meilisearch |
| `meilisearch:clear` | Clear all indexed data |
| `meilisearch:create` | Create indexes with configured settings |
| `meilisearch:delete` | Delete indexes |

## Raw client access

For advanced operations, access the Meilisearch client directly:

```php
use Meilisearch\Client;

class MyService
{
    public function __construct(private Client $client) {}

    public function customOperation(): void
    {
        $index = $this->client->index('movies');
        $stats = $index->getStats();
    }
}
```

## Next steps

> [card] [Full-text search](https://www.meilisearch.com/docs/capabilities/full_text_search/relevancy/relevancy)
>
> Configure ranking and relevancy

> [card] [Filtering](https://www.meilisearch.com/docs/capabilities/filtering_sorting_faceting/getting_started)
>
> Add filters and facets

> [card] [AI-powered search](https://www.meilisearch.com/docs/capabilities/hybrid_search/getting_started)
>
> Add semantic search

> [card] [API reference](https://www.meilisearch.com/docs/reference/api/search/search-with-post)
>
> Explore all search parameters

## Resources

* [meilisearch-symfony on GitHub](https://github.com/meilisearch/meilisearch-symfony)
* [Bundle documentation](https://github.com/meilisearch/meilisearch-symfony/wiki)


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.