# LOLWUT

Displays computer art and the Redis version

## Syntax

```text
LOLWUT [VERSION version]
```


**Available since** 5.0.0

**Group** server

**Arity** -1

**Command flags** READONLY, FAST

## Arguments

```json
[
  {
    "token": "VERSION",
    "name": "version",
    "type": "integer",
    "optional": true
  }
]
```


## Reply schema

```json
{
  "type": "string",
  "description": "String containing the generative computer art, and a text with the Redis version."
}
```
