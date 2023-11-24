using System.Reflection;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace DotNetAstGen.Utils
{
    class IgnorePropertiesResolver : DefaultContractResolver
    {
        private readonly string[] _propsToIgnore =
        {
            "Parent", "ParentTrivia", "ContainsSkippedText", "SyntaxTree", "SpanStart", "IsMissing",
            "IsStructuredTrivia", "HasStructuredTrivia", "ContainsDiagnostics", "ContainsDirectives",
            "HasLeadingTrivia", "HasTrailingTrivia", "ContainsAnnotations", "Span", "FullSpan", "LeadingTrivia",
            "TrailingTrivia"
        };

        private readonly HashSet<string> _ignoreProps;

        public IgnorePropertiesResolver()
        {
            _ignoreProps = new HashSet<string>(this._propsToIgnore);
        }

        protected override JsonProperty CreateProperty(MemberInfo member, MemberSerialization memberSerialization)
        {
            var property = base.CreateProperty(member, memberSerialization);
            if (_ignoreProps.Contains(property.PropertyName ?? ""))
            {
                property.ShouldSerialize = _ => false;
            }

            return property;
        }
    }
}