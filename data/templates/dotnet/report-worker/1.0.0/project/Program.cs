using System.Text.Json;
using Microsoft.Extensions.Options;

var builder = Host.CreateApplicationBuilder(args);
builder.Services.AddOptions<ReportOptions>()
    .Bind(builder.Configuration.GetSection("Reports"))
    .Validate(options => options.IntervalSeconds is >= 1 and <= 3600, "IntervalSeconds must be 1 to 3600.")
    .Validate(options => Directory.Exists(options.InputDirectory), "InputDirectory must exist.")
    .Validate(options => !string.IsNullOrWhiteSpace(options.OutputFile), "OutputFile is required.")
    .ValidateOnStart();
builder.Services.AddHostedService<ReportWorker>();
await builder.Build().RunAsync();

sealed class ReportOptions
{
    public string InputDirectory { get; init; } = "inputs";
    public string OutputFile { get; init; } = "reports/latest.json";
    public int IntervalSeconds { get; init; } = 30;
    public bool RunOnce { get; init; }
}

sealed record DirectoryReport(int Files, long Bytes);

sealed class ReportWorker(IOptions<ReportOptions> settings, IHostApplicationLifetime lifetime,
    ILogger<ReportWorker> log) : BackgroundService
{
    protected override async Task ExecuteAsync(CancellationToken cancellationToken)
    {
        var options = settings.Value;
        using var timer = new PeriodicTimer(TimeSpan.FromSeconds(options.IntervalSeconds));
        do
        {
            var files = Directory.EnumerateFiles(options.InputDirectory, "*.txt");
            var report = new DirectoryReport(0, 0);
            foreach (var path in files)
            {
                cancellationToken.ThrowIfCancellationRequested();
                report = new DirectoryReport(report.Files + 1, report.Bytes + new FileInfo(path).Length);
            }

            var output = Path.GetFullPath(options.OutputFile);
            Directory.CreateDirectory(Path.GetDirectoryName(output)!);
            var temporary = $"{output}.{Guid.NewGuid():N}.tmp";
            try
            {
                await File.WriteAllTextAsync(temporary, JsonSerializer.Serialize(report), cancellationToken);
                File.Move(temporary, output, overwrite: true);
            }
            finally
            {
                File.Delete(temporary);
            }

            log.LogInformation("Wrote report for {FileCount} files ({ByteCount} bytes) to {OutputPath}",
                report.Files, report.Bytes, output);
            if (options.RunOnce)
            {
                lifetime.StopApplication();
                return;
            }
        } while (await timer.WaitForNextTickAsync(cancellationToken));
    }
}
