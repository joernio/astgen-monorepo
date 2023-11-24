using System.Text.RegularExpressions;
using System.Text.Json;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json.Linq;
using Newtonsoft.Json;

namespace DotNetAstGen.Utils
{
    public static class Utilities
    {

        private static ILogger? _logger = Program.LoggerFactory?.CreateLogger("Utilities");

        public static (string kind, int line_start, int line_end, int col_start, int col_end) GetNodeMetadata(
            SyntaxNode node)
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


        public static (string kind, int line_start, int line_end, int col_start, int col_end) GetNodeMetadata(
            SyntaxToken node)
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
            var (kind, lineStart, lineEnd, colStart, colEnd) = GetNodeMetadata(node);
            if (kind == "ast.None")
            {
                return map;
            }

            map.Add("kind", kind);
            map.Add("line_start", lineStart);
            map.Add("line_end", lineEnd);
            map.Add("col_start", colStart);
            map.Add("col_end", colEnd);

            return map;
        }

        public static JObject AddNodeMetadataToMap(SyntaxToken node)
        {
            JObject map = JObject.FromObject(node, new Newtonsoft.Json.JsonSerializer
            {
                ReferenceLoopHandling = ReferenceLoopHandling.Ignore
            });

            var (kind, lineStart, lineEnd, colStart, colEnd) = Utilities.GetNodeMetadata(node);
            if (kind == "ast.None")
            {
                return map;
            }

            map.Add("kind", kind);
            map.Add("line_start", lineStart);
            map.Add("line_end", lineEnd);
            map.Add("col_start", colStart);
            map.Add("col_end", colEnd);

            return map;
        }
    }
}