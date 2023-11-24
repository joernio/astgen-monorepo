using System.Reflection;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

class IgnorePropertiesResolver : DefaultContractResolver
{
    string[] propsToIgnore ={"Parent", "ParentTrivia", "ContainsSkippedText" ,"SyntaxTree", "SpanStart","IsMissing","IsStructuredTrivia","HasStructuredTrivia","ContainsDiagnostics","ContainsDirectives","HasLeadingTrivia","HasTrailingTrivia","ContainsAnnotations","Span","FullSpan","LeadingTrivia","TrailingTrivia"};
    private readonly HashSet<string> _ignoreProps;
    public IgnorePropertiesResolver()
    {
        _ignoreProps = new HashSet<string>(this.propsToIgnore);
    }

    protected override JsonProperty CreateProperty(MemberInfo member, MemberSerialization memberSerialization)
    {

        JsonProperty property = base.CreateProperty(member, memberSerialization);
        if (_ignoreProps.Contains(property.PropertyName ?? ""))
        {
            property.ShouldSerialize = _ => false;
        }
        return property;
    }
}
