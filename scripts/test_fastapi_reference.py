"""Publisher API documentation must survive the static Python importer."""

import ast
import importlib.util
from pathlib import Path
import sys
import unittest

from fastapi_reference import api_description
from reference_rendering import html_from_markdown


spec = importlib.util.spec_from_file_location("fastapi_ingest", Path(__file__).with_name("fetch-docs.py"))
ingest = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = ingest
spec.loader.exec_module(ingest)


class FastAPIReferenceTests(unittest.TestCase):
    def test_site_sponsor_templates_are_omitted_without_losing_tutorial_code(self):
        raw = """# FastAPI

Publisher framework introduction.

## Sponsors { #sponsors }

<!-- sponsors -->
### Keystone Sponsor
{% for sponsor in sponsors.keystone -%}
<div>{{ sponsor.title }}</div>
{% endfor -%}
<!-- /sponsors -->

[Other sponsors](https://fastapi.tiangolo.com/fastapi-people/#sponsors)

## Templates

```html
<a href="{{ url_for('read_item', id=id) }}">Item {{ id }}</a>
{% if item %}Rendered item{% endif %}
```
"""
        result = ingest.fastapi_site_content(raw, "index.md")
        body, markup = html_from_markdown(result, "https://fastapi.tiangolo.com/", "fastapi", {})
        self.assertIn("Publisher framework introduction.", body)
        self.assertIn('id="sponsors"', markup)
        self.assertIn('href="https://fastapi.tiangolo.com/fastapi-people/#sponsors"', markup)
        self.assertNotIn("sponsor.title", body)
        self.assertIn("{{ url_for('read_item', id=id) }}", body)
        self.assertIn("{% if item %}Rendered item{% endif %}", body)
        tutorial = "# Templates\n\n```html\n{{ id }}\n{% for item in items %}{{ item }}{% endfor %}\n```\n"
        self.assertEqual(ingest.fastapi_site_content(tutorial, "advanced/templates.md"), tutorial)

    def test_unresolved_site_templates_fail_in_prose(self):
        with self.assertRaisesRegex(RuntimeError, "Unresolved FastAPI site template"):
            ingest.fastapi_site_content("# People\n\n{% for user in people %}{{ user.name }}{% endfor %}", "people.md")
        with self.assertRaisesRegex(RuntimeError, "sponsor boundaries changed"):
            ingest.fastapi_site_content("# FastAPI\n\nPublisher introduction.", "index.md")

    def test_constructor_parameter_docs_examples_members_and_inheritance(self):
        module = ast.parse('''
class Connection:
    @property
    def headers(self):
        """Publisher inherited headers documentation."""
class App(Connection[State]):
    """Publisher application documentation."""
    def __init__(self, *, title: Annotated[str, Doc("Publisher title explanation.")] = "App"):
        self.state: Annotated[State, Doc("Publisher state explanation.")] = State()
    def get(self, path: Annotated[str, Doc("Publisher path explanation.")]):
        """Publisher method example.

        ```python
        @app.get("/items")
        def items():
            return []
        ```
        """
    def hidden(self):
        """Not selected by the publisher reference."""
    def init_headers(self):
        self.raw_headers = []
''')
        parent, child = module.body
        parent._source_module = child._source_module = "fixture"
        output = api_description("fixture.App", child, {"fixture.Connection": parent}, ["state", "get", "headers", "raw_headers"])
        self.assertIn("@property\ndef headers(self)", output)
        self.assertIn("fixture.App.raw_headers", output)
        with self.assertRaisesRegex(RuntimeError, "fixture.App.missing"):
            api_description("fixture.App", child, {}, ["missing"])
        body, markup = html_from_markdown(output, "https://example.org/reference/app/", "fastapi", {})
        for expected in ("Publisher application documentation.", "Publisher title explanation.",
                         "Publisher state explanation.", "Publisher path explanation.",
                         "Publisher inherited headers documentation.", '@app.get("/items")'):
            self.assertIn(expected, body)
        self.assertNotIn("Not selected", body)
        self.assertIn('id="fixture.App.get"', markup)
        self.assertIn('id="fixture.App.__init__--title"', markup)
        self.assertNotIn("Doc(", body)

    def test_module_reference_includes_model_fields_and_constants(self):
        module = ast.parse('''
HTTP_200_OK = 200
class Model:
    title: Annotated[str, Doc("The publisher field description.")]
''')
        output = api_description("fixture", module)
        self.assertIn("HTTP_200_OK = 200", output)
        self.assertIn("The publisher field description.", output)
        self.assertIn("fixture.Model.title", output)

    def test_qualified_annotated_and_deprecation_text_are_preserved(self):
        node = ast.parse('''
@deprecated("Publisher migration advice.")
async def call(value: typing.Annotated[int, Doc("Parameter documentation.")]) -> Annotated[str, Doc("Result documentation.")]:
    """Call documentation."""
''').body[0]
        output = api_description("fixture.call", node)
        for text in ("async def call(value: int) -> str", "Publisher migration advice.",
                     "Parameter documentation.", "Result documentation."):
            self.assertIn(text, output)


if __name__ == "__main__":
    unittest.main()
