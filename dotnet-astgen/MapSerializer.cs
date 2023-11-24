using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Newtonsoft.Json.Linq;
using System.Data;
using System.Runtime.InteropServices;

public class MapSerializer

{

    public MapSerializer() { }

    public static JArray ProcessSyntaxTokenList(SyntaxTokenList list)
    {
        JArray syntaxTokenList = new();
        foreach (SyntaxToken token in list)
        {
            syntaxTokenList.Add(
                ProcessSyntaxToken(token)
            );
        }
        Console.WriteLine(syntaxTokenList);
        return syntaxTokenList;
    }
    public static JObject ProcessSyntaxToken(SyntaxToken token)
    {
        return Utilities.AddNodeMetadataToMap(token);
    }

    public static void ProcessSyntaxNodeList(dynamic list)
    {
        if (list == null) {
            return;
        }
        foreach (SyntaxNode node in list)
        {
            foreach (var key in node.GetType().GetProperties())
            {
                var valueType = key.GetValue(node)?.GetType() ?? null;
                var value = key.GetValue(node);
                var genericType = (valueType?.IsGenericType ?? false) ? valueType?.GetGenericArguments()[0] : null;

                if (key?.GetValue(node)?.GetType() == typeof(SyntaxToken))
                {
                    Console.WriteLine(ProcessSyntaxToken((SyntaxToken)key.GetValue(node)));
                    // Console.WriteLine(key?.GetValue(node)?.GetType());
                }
                else if (key?.GetValue(node)?.GetType() == typeof(SyntaxNode))
                {
                    Console.WriteLine("SyntaxNode inside SyntaxNode");
                }


                if (genericType?.IsSubclassOf(typeof(SyntaxNode)) ?? false)
                {
                    ProcessSyntaxNodeList(value);
                }
            }

        }
    }

    public static void Helper(dynamic node, JObject map)
    {
        var keys = node.GetType().GetProperties();
        foreach (var key in keys)
        {
            var valueType = key.GetValue(node)?.GetType() ?? null;
            var value = key.GetValue(node);
            if (valueType?.IsPrimitive ?? true)
            {
                Console.WriteLine("Primitive type");
                continue;
            }

            var genericType = (valueType?.IsGenericType ?? false) ? valueType?.GetGenericArguments()[0] : null;
            if (
                genericType?.IsSubclassOf(typeof(CSharpSyntaxNode)) ?? false
            )
            {
                // List of syntax nodes
                Console.WriteLine(value.GetType());
                ProcessSyntaxNodeList(value);
            }

            if (
                valueType == typeof(SyntaxToken)
            )
            {
                Console.WriteLine($"{key}: SyntaxToken type");
            }

        }
    }



    public static List<T> CreateList<T>(params T[] elements)
    {
        return new List<T>(elements);
    }

    public static void SerializeToMap(CompilationUnitSyntax root, Dictionary<string, object> map)
    {
        JObject customMap = JObject.Parse("{}");
        Helper(root, customMap);
    }
}