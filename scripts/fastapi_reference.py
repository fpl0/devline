"""Render publisher Python API declarations and documentation without executing them."""

import ast
import copy
import inspect

from reference_rendering import code_fence


FUNCTIONS = (ast.FunctionDef, ast.AsyncFunctionDef)


def annotation_parts(annotation):
    """Keep Annotated's type in signatures and its publisher Doc text in prose."""
    if (isinstance(annotation, ast.Subscript)
            and ast.unparse(annotation.value).rsplit(".", 1)[-1] == "Annotated"
            and isinstance(annotation.slice, ast.Tuple)):
        parts = annotation.slice.elts
        descriptions = []
        for metadata in parts[1:]:
            if not isinstance(metadata, ast.Call) or not metadata.args:
                continue
            kind = ast.unparse(metadata.func).rsplit(".", 1)[-1]
            text = metadata.args[0]
            if kind in {"Doc", "deprecated"} and isinstance(text, ast.Constant) and isinstance(text.value, str):
                descriptions.append(("**Deprecated**\n\n" if kind == "deprecated" else "")
                                    + inspect.cleandoc(text.value))
        return parts[0], "\n\n".join(descriptions)
    return annotation, ""


def api_parameters(name, node, level):
    original = [*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs,
                node.args.vararg, node.args.kwarg]
    chunks = []
    for argument in original:
        if argument is None:
            continue
        _, description = annotation_parts(argument.annotation)
        if description:
            chunks.append(f"\n{'#' * level} `{argument.arg}` {{#{name}--{argument.arg}}}\n\n{description}\n")
    _, description = annotation_parts(node.returns)
    if description:
        chunks.append(f"\n{'#' * level} Return value\n\n{description}\n")
    return "".join(chunks)


def class_members(node, symbols, visited=None):
    """Resolve inherited members against the two pinned source trees, never imports."""
    visited = set() if visited is None else visited
    if id(node) in visited:
        return {}
    visited.add(id(node))
    members = {}
    module = getattr(node, "_source_module", "")
    for base in reversed(node.bases):
        base_name = ast.unparse(base.value if isinstance(base, ast.Subscript) else base)
        parent = symbols.get(f"{module}.{base_name}") or symbols.get(base_name)
        if isinstance(parent, ast.ClassDef):
            members.update(class_members(parent, symbols, visited))
    for item in node.body:
        if isinstance(item, (*FUNCTIONS, ast.ClassDef)):
            members[item.name] = item
        elif isinstance(item, (ast.Assign, ast.AnnAssign)):
            for target in item.targets if isinstance(item, ast.Assign) else [item.target]:
                if isinstance(target, ast.Name):
                    members[target.id] = item
        if isinstance(item, FUNCTIONS):
            # Instance attributes and their source annotations form part of the
            # published class reference, including attributes initialized by a
            # helper such as Response.init_headers().
            for attribute in ast.walk(item):
                if not isinstance(attribute, (ast.Assign, ast.AnnAssign)):
                    continue
                targets = attribute.targets if isinstance(attribute, ast.Assign) else [attribute.target]
                for target in targets:
                    if isinstance(target, ast.Attribute) and isinstance(target.value, ast.Name) and target.value.id == "self":
                        members.setdefault(target.attr, attribute)
    return members


def api_description(name, node, symbols=None, members=None, level=3):
    symbols = symbols or {}
    description = ""
    extra = ""
    if isinstance(node, FUNCTIONS):
        args = copy.deepcopy(node.args)
        for argument in [*args.posonlyargs, *args.args, *args.kwonlyargs, args.vararg, args.kwarg]:
            if argument:
                argument.annotation, _ = annotation_parts(argument.annotation)
        async_prefix = "async " if isinstance(node, ast.AsyncFunctionDef) else ""
        returns, _ = annotation_parts(node.returns)
        signature = f"{async_prefix}def {node.name}({ast.unparse(args)})" + (f" -> {ast.unparse(returns)}" if returns else "")
        decorators = [ast.unparse(decorator) for decorator in node.decorator_list
                      if ast.unparse(decorator).rsplit(".", 1)[-1] in {"property", "staticmethod", "classmethod"}]
        if decorators:
            signature = "\n".join("@" + decorator for decorator in decorators) + "\n" + signature
        description = ast.get_docstring(node) or ""
        extra = api_parameters(name, node, min(level + 1, 6))
        for decorator in node.decorator_list:
            if (isinstance(decorator, ast.Call) and ast.unparse(decorator.func).endswith("deprecated")
                    and decorator.args and isinstance(decorator.args[0], ast.Constant)):
                description += "\n\n**Deprecated**\n\n" + inspect.cleandoc(str(decorator.args[0].value))
    elif isinstance(node, ast.ClassDef):
        base = ", ".join(map(ast.unparse, node.bases))
        signature = f"class {node.name}" + (f"({base})" if base else "")
        description = ast.get_docstring(node) or ""
        available = class_members(node, symbols)
        constructor = available.get("__init__")
        if constructor:
            extra += api_description(name + ".__init__", constructor, symbols, level=min(level + 1, 5))
        selected = members if members is not None else [key for key in available if not key.startswith("_")]
        for key in selected:
            member = available.get(key)
            if member is None:
                raise RuntimeError(f"Publisher-selected API member is unavailable in the pinned sources: {name}.{key}")
            extra += api_description(name + "." + key, member, symbols, level=min(level + 1, 5))
    elif isinstance(node, ast.Module):
        signature = f"import {name}"
        description = ast.get_docstring(node) or ""
        for item in node.body:
            if isinstance(item, (*FUNCTIONS, ast.ClassDef)) and not item.name.startswith("_"):
                extra += api_description(f"{name}.{item.name}", item, symbols, level=min(level + 1, 5))
            elif isinstance(item, (ast.Assign, ast.AnnAssign)):
                for target in item.targets if isinstance(item, ast.Assign) else [item.target]:
                    if isinstance(target, ast.Name) and not target.id.startswith("_"):
                        extra += api_description(f"{name}.{target.id}", item, symbols, level=min(level + 1, 5))
    elif isinstance(node, ast.AnnAssign):
        annotation, description = annotation_parts(node.annotation)
        signature = name.rsplit(".", 1)[-1] + ": " + ast.unparse(annotation)
        if node.value is not None:
            signature += " = " + ast.unparse(node.value)
    else:
        signature = ast.unparse(node)
    return f"\n{'#' * level} `{name}` {{#{name}}}\n" + code_fence(signature, "python") + "\n" + description + "\n" + extra
