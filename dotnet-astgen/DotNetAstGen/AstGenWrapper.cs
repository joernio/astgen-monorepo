using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;

namespace DotNetAstGen;

public class AstGenWrapper
{
    public AstGenWrapper(string fileName, SyntaxTree tree)
    {
        AstRoot = tree.GetCompilationUnitRoot();
        FileName = fileName;
    }

    public CompilationUnitSyntax AstRoot { get; set; }
    public string FileName { get; set; }
}