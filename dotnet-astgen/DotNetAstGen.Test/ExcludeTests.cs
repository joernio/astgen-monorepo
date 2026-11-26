namespace DotNetAstGen.Test;

public class ExcludeTests
{
    private static readonly string Root = Path.Combine(Path.GetTempPath(), "proj");
    private static readonly string Sep = Path.DirectorySeparatorChar == '\\' ? "[/\\\\]" : "/";

    private static bool IsExcluded(string? pattern, params string[] segments) =>
        Program.IsExcluded(pattern, Root, Path.Combine(new[] { Root }.Concat(segments).ToArray()));

    [Fact]
    public void NoPatternExcludesNothing()
    {
        Assert.False(IsExcluded(null, "Main.cs"));
    }

    [Fact]
    public void ExcludesFileWhoseRelativePathFullyMatches()
    {
        Assert.True(IsExcluded(@"Main\.cs", "Main.cs"));
        Assert.False(IsExcluded(@"Main\.cs", "sub", "Other.cs"));
    }

    [Fact]
    public void ExcludesEverythingBelowMatchingDirectory()
    {
        Assert.True(IsExcluded($"sub{Sep}.*", "sub", "A.cs"));
        Assert.False(IsExcluded($"sub{Sep}.*", "Main.cs"));
    }

    [Fact]
    public void SupportsLookaheads()
    {
        var pattern = $@"(?!sub{Sep}.*$).*\.cs";
        Assert.True(IsExcluded(pattern, "Main.cs"));
        Assert.False(IsExcluded(pattern, "sub", "A.cs"));
    }

    [Fact]
    public void PartialMatchDoesNotExclude()
    {
        Assert.False(IsExcluded("ain", "Main.cs"));
    }

    [Fact]
    public void DoesNotMatchTheAbsolutePath()
    {
        Assert.False(IsExcluded($"{Sep}.*Main\\.cs", "Main.cs"));
    }

    [Fact]
    public void IsCaseSensitive()
    {
        Assert.False(IsExcluded(@"main\.cs", "Main.cs"));
        Assert.True(IsExcluded(@"Main\.cs", "Main.cs"));
    }
}