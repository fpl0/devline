# React quick start

> Integrate a search-as-you-type experience into your React app.

Integrate a search-as-you-type experience into your React app.

## 1. Create a React application

Create your React application using a [Vite](https://vitejs.dev/) template:

```bash
npm create vite@latest my-app -- --template react
```

## 2. Install the library of search components

Navigate to your React app and install `react-instantsearch`, `@meilisearch/instant-meilisearch`, and `instantsearch.css`.

```bash
npm install react-instantsearch @meilisearch/instant-meilisearch instantsearch.css
```

* [React InstantSearch](https://github.com/algolia/instantsearch/): front-end tools to customize your search environment
* [instant-meilisearch](https://github.com/meilisearch/meilisearch-js-plugins/tree/main/packages/instant-meilisearch): Meilisearch client to connect with React InstantSearch
* [instantsearch.css](https://github.com/algolia/instantsearch/tree/master/packages/instantsearch.css) (optional): CSS library to add basic styles to the search components

## 3. Initialize the search client

Use the following URL and API key to connect to a Meilisearch instance containing data from Steam video games.

```jsx
import React from 'react';
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';

const { searchClient } = instantMeiliSearch(
  'https://ms-adf78ae33284-106.lon.meilisearch.io',
  'a63da4928426f12639e19d62886f621130f3fa9ff3c7534c5d179f0f51c4f303'
);
```

## 4. Add the InstantSearch provider

`<InstantSearch>` is the root provider component for the InstantSearch library. It takes two props: the `searchClient` and the [index name](https://www.meilisearch.com/docs/resources/internals/indexes#index-uid).

```jsx
import React from 'react';
import { InstantSearch } from 'react-instantsearch';
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';

const { searchClient } = instantMeiliSearch(
  'https://ms-adf78ae33284-106.lon.meilisearch.io',
  'a63da4928426f12639e19d62886f621130f3fa9ff3c7534c5d179f0f51c4f303'
);

const App = () => (
  <InstantSearch
    indexName="steam-videogames"
    searchClient={searchClient}
  >
  </InstantSearch>
);

export default App
```

## 5. Add a search bar and list search results

Add the `SearchBox` and `InfiniteHits` components inside the `InstantSearch` wrapper component. The Hits component accepts a custom Hit component via the `hitComponent` prop, which allows customizing how each search result is rendered.

Import the CSS library to style the search components.

```jsx
import React from 'react';
import { InstantSearch, SearchBox, InfiniteHits } from 'react-instantsearch';
import { instantMeiliSearch } from '@meilisearch/instant-meilisearch';
import 'instantsearch.css/themes/satellite.css';

const { searchClient } = instantMeiliSearch(
  'https://ms-adf78ae33284-106.lon.meilisearch.io',
  'a63da4928426f12639e19d62886f621130f3fa9ff3c7534c5d179f0f51c4f303'
);

const App = () => (
  <InstantSearch
    indexName="steam-videogames"
    searchClient={searchClient}
  >
    <SearchBox />
    <InfiniteHits hitComponent={Hit} />
  </InstantSearch>
);

const Hit = ({ hit }) => (
  <article key={hit.id}>
    <img src={hit.image} alt={hit.name} />
    <h1>{hit.name}</h1>
    <p>${hit.description}</p>
  </article>
);
export default App
```

> [!TIP]
>
> Use the following CSS classes to add custom styles to your components:
> `.ais-InstantSearch`, `.ais-SearchBox`, `.ais-InfiniteHits-list`, `.ais-InfiniteHits-item`

## 6. Start the app and search as you type

Start the app by running:

```bash
npm run dev
```

Now open your browser and navigate to your React app URL (e.g. `localhost:3000`), and start searching.

![React app search UI with a search bar at the top and search results for a few video games](https://mintcdn.com/meilisearch-6b28dec2/JZ1wsU7CEWrp9Xec/assets/images/react_quick_start/react-qs-search-ui.png?fit=max\&auto=format\&n=JZ1wsU7CEWrp9Xec\&q=85\&s=bd1b370f6f241678c10d73fcabc53f35)

Encountering issues? Check out the code in action in our [live demo](https://codesandbox.io/p/sandbox/eager-dust-f98w2w)!

## Next steps

Want to search through your own data? [Create a project](https://cloud.meilisearch.com) in the Meilisearch Dashboard. Check out our [getting started guide](https://www.meilisearch.com/docs/getting_started/overview) for step-by-step instructions.


This documentation is built and hosted on [Mintlify](https://mintlify.com), a developer documentation platform.