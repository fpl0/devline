# INFO

Returns information and statistics about the server.

## Syntax

```text
INFO [section [section ...]]
```


**Available since** 1.0.0

**Group** server

**Time complexity** O(1)

**Arity** -1

**ACL categories** DANGEROUS

**Command flags** LOADING, STALE, SENTINEL

## History

- 7.0.0 — Added support for taking multiple section arguments.

## Arguments

```json
[
  {
    "name": "section",
    "type": "string",
    "multiple": true,
    "optional": true
  }
]
```


## Reply schema

```json
{
  "description": "A map of info fields, one field per line in the form of <field>:<value> where the value can be a comma separated map like <key>=<val>. Also contains section header lines starting with `#` and blank lines.",
  "type": "string"
}
```
