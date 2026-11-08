using System.Globalization;
using System.Text.Json;

if (args.Length == 0 || args is ["--help"] or ["-h"])
{
    Console.WriteLine("""
        json-cli format <file|->
        json-cli get <JSON pointer> <file|->

        Use - to read standard input. Example pointer: /books/0/title.
        """);
    return 0;
}

if (args is not ["format", _] and not ["get", _, _])
{
    Console.Error.WriteLine("Expected format <file|-> or get <JSON pointer> <file|->. Try --help.");
    return 2;
}

try
{
    using var input = args[^1] == "-" ? Console.OpenStandardInput() : File.OpenRead(args[^1]);
    using var document = JsonDocument.Parse(input);
    var value = args[0] == "get" ? ResolvePointer(document.RootElement, args[1]) : document.RootElement;
    Console.WriteLine(JsonSerializer.Serialize(value, new JsonSerializerOptions { WriteIndented = true }));
    return 0;
}
catch (Exception error) when (error is JsonException or IOException or UnauthorizedAccessException or FormatException)
{
    Console.Error.WriteLine($"json-cli: {error.Message}");
    return 1;
}

static JsonElement ResolvePointer(JsonElement value, string pointer)
{
    if (pointer.Length == 0)
        return value;
    if (!pointer.StartsWith('/'))
        throw new FormatException("A JSON pointer must be empty or start with /.");

    foreach (var token in pointer[1..].Split('/'))
    {
        var key = DecodeToken(token);
        value = value.ValueKind switch
        {
            JsonValueKind.Object when value.TryGetProperty(key, out var property) => property,
            JsonValueKind.Array when TryArrayIndex(key, value.GetArrayLength(), out var index) => value[index],
            _ => throw new FormatException($"Pointer segment '{key}' was not found.")
        };
    }

    return value;
}

static string DecodeToken(string token)
{
    for (var i = 0; i < token.Length; i++)
    {
        if (token[i] == '~' && (++i == token.Length || token[i] is not ('0' or '1')))
            throw new FormatException("A JSON pointer uses ~0 for ~ and ~1 for /.");
    }

    return token.Replace("~1", "/").Replace("~0", "~");
}

static bool TryArrayIndex(string key, int length, out int index)
{
    return int.TryParse(key, NumberStyles.None, CultureInfo.InvariantCulture, out index)
        && (key.Length == 1 || key[0] != '0')
        && index >= 0 && index < length;
}
