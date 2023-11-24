using System.Text.RegularExpressions;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Newtonsoft.Json.Linq;
using Newtonsoft.Json;

public class Utilities
{
    public static List<string> GetAllCSFilesInDirectory(string dirPath, List<string> result)
    {
        try
        {
            foreach (string dir in Directory.GetDirectories(dirPath))
            {
                foreach (string file in Directory.GetFiles(dir))
                {
                    if (new Regex("^.*\\.cs$").Matches(file).Count >= 1)
                    {
                        result.Add(file);
                    }
                }
                GetAllCSFilesInDirectory(dir, result);
            }
        }
        catch (System.Exception except)
        {
            Console.WriteLine(except.Message);
        }
        return result;
    }

    public static (string kind, int line_start, int line_end, int col_start, int col_end) GetNodeMetadata(SyntaxNode node)
    {
        FileLinePositionSpan span = node.SyntaxTree.GetLineSpan(node.Span);
        return (
            $"ast.{node.Kind()}",
            span.StartLinePosition.Line,
            span.EndLinePosition.Line,
            span.StartLinePosition.Character,
            span.EndLinePosition.Character
        );
    }


    public static (string kind, int line_start, int line_end, int col_start, int col_end) GetNodeMetadata(SyntaxToken node)
    {
        try
        {
            FileLinePositionSpan span = node.SyntaxTree.GetLineSpan(node.Span);
            return (
                $"ast.{node.Kind()}",
                span.StartLinePosition.Line,
                span.EndLinePosition.Line,
                span.StartLinePosition.Character,
                span.EndLinePosition.Character
            );
        }
        catch
        {
            return (
            $"ast.{node.Kind()}",
            -1,
            -1,
            -1,
            -1
        );
        }

    }

    public static JObject AddNodeMetadataToMap(SyntaxNode node)
    {
        JObject map = JObject.FromObject(node, new Newtonsoft.Json.JsonSerializer
        {
            ReferenceLoopHandling = ReferenceLoopHandling.Ignore
        });
        var (kind, line_start, line_end, col_start, col_end) = Utilities.GetNodeMetadata(node);
        if (kind == "ast.None")
        {
            return map;
        }
        map.Add("kind", kind);
        map.Add("line_start", line_start);
        map.Add("line_end", line_end);
        map.Add("col_start", col_start);
        map.Add("col_end", col_end);

        return map;
    }

    public static JObject AddNodeMetadataToMap(SyntaxToken node)
    {
        JObject map = JObject.FromObject(node, new Newtonsoft.Json.JsonSerializer
        {
            ReferenceLoopHandling = ReferenceLoopHandling.Ignore
        });

        var (kind, line_start, line_end, col_start, col_end) = Utilities.GetNodeMetadata(node);
        if (kind == "ast.None")
        {
            return map;
        }
        map.Add("kind", kind);
        map.Add("line_start", line_start);
        map.Add("line_end", line_end);
        map.Add("col_start", col_start);
        map.Add("col_end", col_end);

        return map;
    }


}