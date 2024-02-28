using Newtonsoft.Json.Serialization;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Newtonsoft.Json;

namespace DotNetAstGen
{
    public class SyntaxMetaDataProvider : IValueProvider
    {
        public object GetValue(object target)
        {
            return target.GetType().IsAssignableTo(typeof(SyntaxNode))
                ? GetNodeMetadata((SyntaxNode)target)
                : new SyntaxMetaData();
        }

        private static SyntaxMetaData GetNodeMetadata(SyntaxNode node)
        {
            var span = node.SyntaxTree.GetLineSpan(node.Span);
            return new SyntaxMetaData(
                $"ast.{node.Kind()}",
                span.StartLinePosition.Line,
                span.EndLinePosition.Line,
                span.StartLinePosition.Character,
                span.EndLinePosition.Character,
                node.GetText().ToString().Trim()
            );
        }

        public void SetValue(object target, object? value)
        {
            // ignore
        }

        public static JsonProperty CreateMetaDataProperty()
        {
            return new JsonProperty
            {
                PropertyName = "MetaData",
                PropertyType = typeof(SyntaxMetaData),
                DeclaringType = typeof(SyntaxNode),
                ValueProvider = new SyntaxMetaDataProvider(),
                AttributeProvider = null,
                Readable = true,
                Writable = false,
                ShouldSerialize = _ => true
            };
        }
    }

    public class SyntaxMetaData
    {
        public SyntaxMetaData()
        {
        }

        public SyntaxMetaData(string kind, int lineStart, int lineEnd, int columnStart, int columnEnd, string code)
        {
            Kind = kind;
            LineStart = lineStart;
            LineEnd = lineEnd;
            ColumnStart = columnStart;
            ColumnEnd = columnEnd;
            Code = code;
        }

        public string Kind { get; set; } = "ast.None";
        public int LineStart { get; set; } = -1;
        public int LineEnd { get; set; } = -1;
        public int ColumnStart { get; set; } = -1;
        public int ColumnEnd { get; set; } = -1;
        public string Code { get; set; } = "<empty>";

        public override string ToString()
        {
            return JsonConvert.SerializeObject(this);
        }
    }
}