using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Newtonsoft.Json;
using System.Reflection;


class Program
{
    static void Main(string[] args)
    {

        string dirPath = "/Users/karanbatavia/Privado/repos/dotnet/Ocelot";
        List<string> files = Utilities.GetAllCSFilesInDirectory(dirPath, new List<string>());

        const string programText = @"using System;
using System.Json;
namespace HelloWorld
{
  class Program
  {
    static void Main(string[] args)
    {

        var a = 10;
      Console.WriteLine(a + 10);    
    }
  }
}";

        SyntaxTree tree = CSharpSyntaxTree.ParseText(programText);
        CompilationUnitSyntax root = tree.GetCompilationUnitRoot();
        Type rootType = root.GetType();
        IList<PropertyInfo> props = new List<PropertyInfo>(rootType.GetProperties());
        string jsonString = JsonConvert.SerializeObject(root, Formatting.Indented, new JsonSerializerSettings
        {
            ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
            ContractResolver = new IgnorePropertiesResolver(), // Comment this to see the unfiltered parser output
        });
        File.WriteAllText("./sample.json", jsonString);
    }
}