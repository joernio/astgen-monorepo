using System.Reflection;
using System.Text.RegularExpressions;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;

namespace DotNetAstGen.Utils
{
    internal class IgnorePropertiesResolver : DefaultContractResolver
    {
        private static readonly ILogger? Logger = Program.LoggerFactory?.CreateLogger("IgnorePropertiesResolver");


        private readonly HashSet<string> _propsToAllow = new(new[]
        {
            "Value", "Externs", "Usings", "Name", "Identifier", "Left", "Right", "Members", "ConstraintClauses",
            "Alias", "NamespaceOrType", "Arguments", "Expression", "Declaration", "ElementType", "Initializer", "Else",
            "Condition", "Statement", "Statements", "Variables", "WhenNotNull", "AllowsAnyExpression", "Expressions",
            "Modifiers", "ReturnType", "IsUnboundGenericName", "Default", "IsConst", "Parameters", "Types",
            "ExplicitInterfaceSpecifier", "Text", "Length", "Location"
        });

        private readonly List<string> _regexToAllow = new(new[]
        {
            ".*Token$", ".*Keyword$", ".*Lists?$", ".*Body$", "(Span)?Start"
        });

        private readonly List<string> _regexToIgnore = new(new[]
        {
            ".*(Semicolon|Brace|Bracket|EndOfFile|Paren|Dot)Token$"
        });

        private bool MatchesAllow(string input)
        {
            return _regexToAllow.Any(regex => Regex.IsMatch(input, regex));
        }

        private bool MatchesIgnore(string input)
        {
            return _regexToIgnore.Any(regex => Regex.IsMatch(input, regex));
        }

        protected override JsonProperty CreateProperty(MemberInfo member, MemberSerialization memberSerialization)
        {
            var property = base.CreateProperty(member, memberSerialization);
            var propertyName = property.PropertyName ?? "";
            var shouldSerialize = propertyName != "" &&
                                  (_propsToAllow.Contains(propertyName) || MatchesAllow(propertyName)) &&
                                  !MatchesIgnore(propertyName);
            Logger?.LogDebug(shouldSerialize ? $"Allowing {propertyName}" : $"Ignoring {propertyName}");
            property.ShouldSerialize = _ => shouldSerialize;
            return property;
        }
    }
}