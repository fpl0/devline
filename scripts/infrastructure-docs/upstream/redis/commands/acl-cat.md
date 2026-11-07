# ACL CAT

Lists the ACL categories, or the commands inside a category.

## Syntax

```text
ACL CAT [category]
```


**Available since** 6.0.0

**Group** server

**Time complexity** O(1) since the categories and commands are a fixed set.

**Arity** -2

**Command flags** NOSCRIPT, LOADING, STALE, SENTINEL

## Arguments

```json
[
  {
    "name": "category",
    "type": "string",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "anyOf": [
    {
      "type": "array",
      "description": "In case `category` was not given, a list of existing ACL categories",
      "items": {
        "type": "string"
      }
    },
    {
      "type": "array",
      "description": "In case `category` was given, list of commands that fall under the provided ACL category",
      "items": {
        "type": "string"
      }
    }
  ]
}
```
