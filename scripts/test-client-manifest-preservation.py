#!/usr/bin/env python3
"""Regress manifest source preservation when adding only selected dependencies."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import xml.etree.ElementTree as ET
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('manifest_edits', ROOT/'data/service-clients/manifest_edits.py')
module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
with tempfile.TemporaryDirectory(prefix='devline-manifest-proof-') as temporary:
    base = Path(temporary)
    for body in ['<dependencies/>', '<dependencies><!-- Keep our existing dependency policy --><dependency><groupId>old</groupId><artifactId>client</artifactId><version>1</version></dependency></dependencies>', '']:
        path = base/'pom.xml'
        original = '<?xml version="1.0"?>\r\n<!-- Copyright Example -->\r\n<project xmlns="http://maven.apache.org/POM/4.0.0"><modelVersion>4.0.0</modelVersion>'+body+'<!-- Keep private registry instructions --></project>'
        path.write_bytes(original.encode())
        module.add_maven_dependencies(path, {'org.postgresql:postgresql': {'version':'42.7.14'}})
        result=path.read_bytes().decode()
        assert '<!-- Copyright Example -->' in result and '<!-- Keep private registry instructions -->' in result
        if 'dependency policy' in original: assert body in result.replace('\n','') or '<!-- Keep our existing dependency policy -->' in result
        assert 'old</groupId>' not in original or '<groupId>old</groupId><artifactId>client</artifactId><version>1</version>' in result
        assert result.count('<artifactId>postgresql</artifactId>') == 1
        before=path.read_bytes();module.add_maven_dependencies(path, {'org.postgresql:postgresql': {'version':'42.7.14'}});assert path.read_bytes()==before
        ET.fromstring(path.read_bytes())
    for definition in ['def project, do: [app: :example, version: "0.1.0", deps: deps()] # keep end comment', 'def project do\n    # Keep dependency rationale.\n    [app: :example, version: "0.1.0", deps: deps()]\n  end']:
        path=base/'mix.exs'
        original='# Copyright Example\ndefmodule Example.MixProject do\n  use Mix.Project\n  '+definition+'\n  defp deps, do: [{:jason, "== 1.4.5"}] # keep original\nend\n'
        path.write_text(original)
        for package in ['postgrex=0.22.4','amqp=4.3.0']:
            subprocess.run(['elixir',str(ROOT/'data/service-clients/add-mix-clients.exs'),str(path),package],check=True)
        result=path.read_text()
        assert '# Copyright Example' in result and definition in result and '{:jason, "== 1.4.5"}' in result and '# keep original' in result
        assert 'defoverridable' not in result
        subprocess.run(['elixir','-e','Mix.start(); Code.require_file(hd(System.argv())); deps = Example.MixProject.project()[:deps]; unless Enum.sort(deps) == Enum.sort([jason: "== 1.4.5", postgrex: "== 0.22.4", amqp: "== 4.3.0"]), do: raise(inspect(deps))',str(path)],check=True)
    path=base/'mix.exs'
    original='# Keep complex configuration\ndefmodule Example.MixProject do\n  use Mix.Project\n  def project, do: [app: :example, version: "0.1.0", deps: base_deps()]\n  defp base_deps, do: [{:jason, "== 1.4.5"}] # preserve computed function\nend\n'
    path.write_text(original)
    subprocess.run(['elixir',str(ROOT/'data/service-clients/add-mix-clients.exs'),str(path),'amqp=4.3.0'],check=True)
    result=path.read_text()
    assert 'defoverridable project: 0' in result and 'defp base_deps, do: [{:jason, "== 1.4.5"}] # preserve computed function' in result
    subprocess.run(['elixir','-e','Mix.start(); Code.require_file(hd(System.argv())); unless Example.MixProject.project()[:deps] == [jason: "== 1.4.5", amqp: "== 4.3.0"], do: raise("wrong deps")',str(path)],check=True)
print('PASS: Maven comments, original dependency text and idempotency; Mix comments, original functions and repeated additive dependency behavior')
