using System.Reflection;
using System.Text.RegularExpressions;
using Microsoft.CodeAnalysis;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace DotNetAstGen
{
    internal class SyntaxNodePropertiesResolver : DefaultContractResolver
    {
        private static readonly ILogger? Logger = Program.LoggerFactory?.CreateLogger("SyntaxNodePropertiesResolver");

        private readonly HashSet<string> _propsToAllow = new(new[]
        {
            "Value", "Usings", "Name", "Identifier", "Left", "Right", "Members", "ConstraintClauses",
            "Alias", "NamespaceOrType", "Arguments", "Expression", "Declaration", "ElementType", "Initializer", "Else",
            "Condition", "Statement", "Statements", "Variables", "WhenNotNull", "AllowsAnyExpression", "Expressions",
            "Modifiers", "ReturnType", "IsUnboundGenericName", "Default", "IsConst", "Types",
            "ExplicitInterfaceSpecifier", "MetaData", "Kind", "AstRoot", "FileName", "Code", "Operand", "Block",
            "Catches", "Finally", "Keyword", "Incrementors", "Sections", "Pattern", "Labels", "Elements" ,"WhenTrue",
            "WhenFalse", "Initializers", "NameEquals", "Contents", "Attributes"
        });

        private readonly List<string> _regexToAllow = new(new[]
        {
            ".*Token$", ".*Lists?$", ".*Body$", "(Line|Column)(Start|End)", ".*Type$", "Parameters?",
        });

        private readonly List<string> _regexToIgnore = new(new[]
        {
            ".*(Semicolon|Brace|Bracket|EndOfFile|Paren|Dot)Token$",
            "(Unsafe|Global|Static|Using)Keyword"
        });

        private bool MatchesAllow(string input)
        {
            return _regexToAllow.Any(regex => Regex.IsMatch(input, regex));
        }

        private bool MatchesIgnore(string input)
        {
            return _regexToIgnore.Any(regex => Regex.IsMatch(input, regex));
        }

        protected override IList<JsonProperty> CreateProperties(Type type, MemberSerialization memberSerialization)
        {
            var properties = base.CreateProperties(type, memberSerialization);
            var isSyntaxNode = type.IsAssignableTo(typeof(SyntaxNode));
            if (!isSyntaxNode) return properties;
            properties.Add(SyntaxMetaDataProvider.CreateMetaDataProperty());
            return properties;
        }

        protected override JsonProperty CreateProperty(MemberInfo member, MemberSerialization memberSerialization)
        {
            var property = base.CreateProperty(member, memberSerialization);
            var propertyName = property.PropertyName ?? "";
            // var shouldSerialize = propertyName != "";
            var shouldSerialize = propertyName != ""
            &&
                                  (_propsToAllow.Contains(propertyName) || MatchesAllow(propertyName)) &&
                                  !MatchesIgnore(propertyName);
            Logger?.LogDebug(shouldSerialize ? $"Allowing {propertyName}" : $"Ignoring {propertyName}");
            property.ShouldSerialize = _ => shouldSerialize;
            return property;
        }
    }
}