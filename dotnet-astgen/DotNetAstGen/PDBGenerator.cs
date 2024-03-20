using ICSharpCode.Decompiler;
using ICSharpCode.Decompiler.CSharp;
using ICSharpCode.Decompiler.CSharp.ProjectDecompiler;
using ICSharpCode.Decompiler.DebugInfo;
using ICSharpCode.Decompiler.Disassembler;
using ICSharpCode.Decompiler.Metadata;
using ICSharpCode.Decompiler.Solution;
using ICSharpCode.Decompiler.TypeSystem;
using ICSharpCode.ILSpyX.PdbProvider;

using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using Microsoft.Extensions.Logging;

namespace DotNetAstGen
{
    public class PDBGenerator
    {
        public static ILoggerFactory? LoggerFactory;
        private static ILogger<PDBGenerator>? _logger;

        private static DecompilerSettings GetSettings(PEFile module)
        {
            return new DecompilerSettings(ICSharpCode.Decompiler.CSharp.LanguageVersion.Latest)
            {
                ThrowOnAssemblyResolveErrors = false,
                RemoveDeadCode = false,
                RemoveDeadStores = false,
                UseSdkStyleProjectFormat = WholeProjectDecompiler.CanUseSdkStyleProjectFormat(module),
                UseNestedDirectoriesForNamespaces = false,
            };
        }
        private static CSharpDecompiler GetDecompiler(string assemblyFileName)
        {
            var module = new PEFile(assemblyFileName);
            var resolver = new UniversalAssemblyResolver(assemblyFileName, false, module.Metadata.DetectTargetFrameworkId());
            return new CSharpDecompiler(assemblyFileName, resolver, GetSettings(module));
        }
        public void GeneratePDBforDLLFile(string dllFileName, string pdbFileName)
        {
            using ILoggerFactory factory = Microsoft.Extensions.Logging.LoggerFactory.Create(builder => builder.AddConsole());
            _logger = factory.CreateLogger<PDBGenerator>();

            var module = new PEFile(dllFileName,
                new FileStream(dllFileName, FileMode.Open, FileAccess.Read),
                PEStreamOptions.PrefetchEntireImage,
                metadataOptions: MetadataReaderOptions.None);

            if (!PDBWriter.HasCodeViewDebugDirectoryEntry(module))
            {
                _logger?.LogWarning($"Cannot create PDB file for {dllFileName}, because it does not contain a PE Debug Directory Entry of type 'CodeView'. Skipping...");
            }
            else
            {
                using (FileStream stream = new FileStream(pdbFileName, FileMode.OpenOrCreate, FileAccess.Write))
                {
                    var decompiler = GetDecompiler(dllFileName);
                    PDBWriter.WritePdb(module, decompiler, GetSettings(module), stream);
                }
            }
        }
    }
}