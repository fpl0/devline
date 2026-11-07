#!/usr/bin/env python3
"""Check .NET adapter file-change polling and explicit overrides without network access."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def main():
    repo = Path(__file__).resolve().parents[1]
    tool = shutil.which('dotnet')
    sandbox = Path('/usr/bin/sandbox-exec')
    if not tool or not sandbox.is_file():
        raise SystemExit('Install a .NET 10 SDK and run this regression on macOS.')
    policy = '(version 1)(allow default)(deny network-outbound)'
    with tempfile.TemporaryDirectory(prefix='devline-file-watcher-') as temporary:
        root = Path(temporary)
        project = root / 'project'
        project.mkdir()
        (root / 'home').mkdir()
        # No external packages: the ASP.NET shared framework supplies the provider.
        (project / 'Watcher.csproj').write_text('''<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings></PropertyGroup>
  <ItemGroup><FrameworkReference Include="Microsoft.AspNetCore.App" /></ItemGroup>
</Project>
''')
        (project / 'Program.cs').write_text('''using Microsoft.Extensions.FileProviders;
using System.Net.Sockets;

try {
    using var socket = new Socket(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
    await socket.ConnectAsync("1.1.1.1", 443).WaitAsync(TimeSpan.FromSeconds(2));
    throw new Exception("Outbound network unexpectedly reachable");
} catch (SocketException error) when (error.SocketErrorCode == SocketError.AccessDenied) { }

using var provider = new PhysicalFileProvider(Directory.GetCurrentDirectory());
var expected = bool.Parse(args[0]);
if (provider.UsePollingFileWatcher != expected || provider.UseActivePolling != expected)
    throw new Exception("The adapter did not propagate the expected file-watcher mode");
if (expected) {
    await File.WriteAllTextAsync("watched.json", "before");
    var changed = new TaskCompletionSource(TaskCreationOptions.RunContinuationsAsynchronously);
    using var subscription = provider.Watch("watched.json").RegisterChangeCallback(_ => changed.TrySetResult(), null);
    await File.WriteAllTextAsync("watched.json", "after change");
    await changed.Task.WaitAsync(TimeSpan.FromSeconds(12));
    Console.WriteLine("PASS: polling delivers actual file-change callbacks with outbound network denied");
} else {
    Console.WriteLine("PASS: explicit false keeps the native file-watcher setting");
}
''')
        cache = project / '.devline-offline'
        (cache / 'dotnet/packages').mkdir(parents=True)
        config = project / 'NuGet.Config'
        config.write_text('<configuration><packageSources><clear /></packageSources></configuration>')
        env = os.environ.copy()
        env.pop('DOTNET_USE_POLLING_FILE_WATCHER', None)
        env.update(HOME=str(root / 'home'), DOTNET_CLI_HOME=str(root / 'home'),
                   NUGET_PACKAGES=str(cache / 'dotnet/packages'), DOTNET_CLI_TELEMETRY_OPTOUT='1',
                   DOTNET_GENERATE_ASPNET_CERTIFICATE='false', DOTNET_NOLOGO='1',
                   DOTNET_EnableDiagnostics='0', DOTNET_CLI_USE_MSBUILD_SERVER='0',
                   MSBUILDDISABLENODEREUSE='1', UseSharedCompilation='false')
        boundary = [str(sandbox), '-p', policy]
        subprocess.run(boundary + [tool, 'restore', '--use-lock-file', '--configfile', str(config), '-p:NuGetAudit=false'],
                       cwd=project, env=env, check=True, timeout=60)
        command = boundary + ['/bin/sh', str(repo / 'data/project-adapters/dotnet.sh'),
                              'run', str(project), str(cache), tool, 'dotnet', 'run',
                              '--no-restore', '-p:UseSharedCompilation=false', '--']
        subprocess.run(command + ['true'], cwd=project, env=env, check=True, timeout=60)
        env['DOTNET_USE_POLLING_FILE_WATCHER'] = 'false'
        subprocess.run(command + ['false'], cwd=project, env=env, check=True, timeout=60)


if __name__ == '__main__':
    main()
