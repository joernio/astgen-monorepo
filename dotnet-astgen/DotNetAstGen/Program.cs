using System.Diagnostics;
using System.Text;
using System.Text.RegularExpressions;
using CommandLine;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json;
using Mono.Cecil;
using Mono.Cecil.Rocks;

namespace DotNetAstGen
{
    public class MethodInfo
    {
        public string? name { get; set; }
        public string? returnType { get; set; }
        public List<List<string>>? parameterTypes { get; set; }
        public bool isStatic { get; set; }
    }

    public class ClassInfo
    {
        public string? name { get; set; }
        public List<MethodInfo>? methods { get; set; }
        public List<object>? fields { get; set; }
    }

    internal class Program
    {
        public static ILoggerFactory? LoggerFactory;
        private static ILogger<Program>? _logger;

        public static void Main(string[] args)
        {
            Parser.Default.ParseArguments<Options>(args)
                .WithParsed(options =>
                {
                    LoggerFactory = Microsoft.Extensions.Logging.LoggerFactory.Create(builder =>
                    {
                        builder
                            .ClearProviders()
                            .AddDebug()
                            .AddSimpleConsole(consoleOptions =>
                            {
                                consoleOptions.IncludeScopes = false;
                                consoleOptions.SingleLine = true;
                            });

                        if (options.Debug)
                        {
                            builder.SetMinimumLevel(LogLevel.Debug);
                        }
                    });

                    _logger = LoggerFactory.CreateLogger<Program>();
                    _logger.LogDebug("Show verbose output.");

                    // Handle .cs
                    _ParseSourceCode(
                        options.InputFilePath,
                        new DirectoryInfo(options.OutputDirectory),
                        options.ExclusionRegex);

                    // Handle DLLs
                    _ParseByteCode(options.InputFilePath,
                        new DirectoryInfo(options.OutputDirectory),
                        options.ExclusionRegex);
                });
        }

        private static void _ParseSourceCode(string inputPath, DirectoryInfo rootOutputPath, string? exclusionRegex)
        {
            if (!rootOutputPath.Exists)
            {
                rootOutputPath.Create();
            }

            if (Directory.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing directory {dirName}", inputPath);
                var rootDirectory = new DirectoryInfo(inputPath);
                foreach (var inputFile in new DirectoryInfo(inputPath).EnumerateFiles("*.cs",
                             SearchOption.AllDirectories))
                {
                    _AstForFile(rootDirectory, rootOutputPath, inputFile, exclusionRegex);
                }
            }
            else if (File.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing file {fileName}", inputPath);
                var file = new FileInfo(inputPath);
                Debug.Assert(file.Directory != null, "Given file has a null parent directory!");
                _AstForFile(file.Directory, rootOutputPath, file, exclusionRegex);
            }
            else
            {
                _logger?.LogError("The path {inputPath} does not exist!", inputPath);
                Environment.Exit(1);
            }

            _logger?.LogInformation("AST generation for `.cs` files is complete");
        }

        private static void _AstForFile(
            FileSystemInfo rootInputPath,
            FileSystemInfo rootOutputPath,
            FileInfo filePath,
            string? exclusionRegex)
        {
            var fullPath = filePath.FullName;
            if (exclusionRegex != null && Regex.IsMatch(fullPath, exclusionRegex))
            {
                _logger?.LogInformation("Skipping file: {filePath}", fullPath);
                return;
            }

            _logger?.LogInformation("Parsing file: {filePath}", fullPath);

            try
            {
                using var streamReader = new StreamReader(fullPath, Encoding.UTF8);
                var programText = streamReader.ReadToEnd();
                var tree = CSharpSyntaxTree.ParseText(programText);
                var diagnostics = new List<Diagnostic>(tree.GetDiagnostics());
                var errorWhileParsing = false;
                foreach (var diagnostic in diagnostics)
                {
                    switch (diagnostic.Severity)
                    {
                        case DiagnosticSeverity.Warning:
                            _logger?.LogWarning(diagnostic.ToString());
                            break;
                        case DiagnosticSeverity.Error:
                            _logger?.LogError(diagnostic.ToString());
                            errorWhileParsing = true;
                            break;
                    }
                }

                if (errorWhileParsing)
                {
                    _logger?.LogError("Error(s) encountered while parsing: {filePath}", fullPath);
                }
                else
                {
                    _logger?.LogInformation("Successfully parsed: {filePath}", fullPath);
                    var astGenResult = new AstGenWrapper(fullPath, tree);
                    var jsonString = JsonConvert.SerializeObject(astGenResult, Formatting.Indented,
                        new JsonSerializerSettings
                        {
                            ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
                            ContractResolver =
                                new SyntaxNodePropertiesResolver() // Comment this to see the unfiltered parser output
                        });
                    var outputName = Path.Combine(filePath.DirectoryName ?? "./",
                            $"{Path.GetFileNameWithoutExtension(fullPath)}.json")
                        .Replace(rootInputPath.FullName, rootOutputPath.FullName);

                    // Create dirs if they do not exist
                    var outputParentDir = Path.GetDirectoryName(outputName);
                    if (outputParentDir != null)
                    {
                        Directory.CreateDirectory(outputParentDir);
                    }

                    File.WriteAllText(outputName, jsonString);
                }
            }
            catch (Exception e)
            {
                _logger?.LogError("Error encountered while parsing '{filePath}': {errorMsg}", fullPath, e.Message);
            }
        }

        private static void _ParseByteCode(string inputPath, DirectoryInfo rootOutputPath, string? exclusionRegex)
        {
            if (!rootOutputPath.Exists)
            {
                rootOutputPath.Create();
            }

            if (Directory.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing directory {dirName}", inputPath);

                foreach (var inputFile in new DirectoryInfo(inputPath).EnumerateFiles("*.dll",
                             SearchOption.AllDirectories))
                {
                    _logger?.LogInformation("Parsing file {fileName}", inputPath);
                    _SummaryForDLLFile(inputFile, exclusionRegex);
                }
            }
            else if (File.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing file {fileName}", inputPath);
                var file = new FileInfo(inputPath);
                Debug.Assert(file.Directory != null, "Given file has a null parent directory!");
                _SummaryForDLLFile(file, exclusionRegex);
            }
            else
            {
                _logger?.LogError("The path {inputPath} does not exist!", inputPath);
                Environment.Exit(1);
            }

            _logger?.LogInformation("Bytecode summary generation complete");
        }

        private static void _SummaryForDLLFile(FileInfo filePath, string? exclusionRegex)
        {
            var fullPath = filePath.FullName;
            if (exclusionRegex != null && Regex.IsMatch(fullPath, exclusionRegex))
            {
                _logger?.LogInformation("Skipping file: {filePath}", fullPath);
                return;
            }

            var jsonName = Path.Combine(filePath.DirectoryName ?? "./",
                $"{Path.GetFileNameWithoutExtension(fullPath)}_Symbols.json");

            ProcessDll(filePath, jsonName);
        }

        static void ProcessDll(FileInfo dllFile, string jsonPath)
        {
            try
            {
                var dllPath = dllFile.FullName;
                var pdbPath = Path.Combine(dllFile.DirectoryName ?? "./",
                    $"{Path.GetFileNameWithoutExtension(dllPath)}.pdb");

                // check if PDB exists
                if (!File.Exists(pdbPath))
                {
                    _logger?.LogInformation("PDB not found! trying to generate PDB locally for: {filePath}", dllPath);
                    PDBGenerator pdbgen = new PDBGenerator();
                    pdbgen.GeneratePDBforDLLFile(dllPath, pdbPath);
                }

                // check again
                if (!File.Exists(pdbPath))
                {
                    _logger?.LogWarning("{}.dll does not have an accompanying PDB file, skipping...",
                        Path.GetFileNameWithoutExtension(dllPath));
                }
                else
                {
                    var p = new ReaderParameters
                    {
                        ReadSymbols = true
                    };

                    var classInfoList = new List<ClassInfo>();

                    using var x = AssemblyDefinition.ReadAssembly(dllPath, p);
                    var typeFilter = new Regex("^(<PrivateImplementationDetails>|<Module>|.*AnonymousType|.*\\/).*",
                        RegexOptions.IgnoreCase);
                    var methodFilter = new Regex("^.*\\.(ctor|cctor)", RegexOptions.IgnoreCase);

                    foreach (var typ in x.MainModule.GetAllTypes().DistinctBy(t => t.FullName).Where(t => t.Name != null)
                                 .Where(t => !typeFilter.IsMatch(t.FullName)))
                    {
                        var classInfo = new ClassInfo();
                        var methodInfoList = new List<MethodInfo>();

                        foreach (var method in typ.Methods.Where(m => !methodFilter.IsMatch(m.Name)).Where(m => m.IsPublic))
                        {
                            var methodInfo = new MethodInfo
                            {
                                name = method.Name,
                                returnType = method.ReturnType.ToString(),
                                isStatic = method.IsStatic
                            };
                            var parameterTypesList = method.Parameters
                                .Select(param => (List<string>)[param.Name, param.ParameterType.FullName]).ToList();

                            methodInfo.parameterTypes = parameterTypesList;
                            methodInfoList.Add(methodInfo);
                        }

                        classInfo.methods = methodInfoList;
                        classInfo.fields = [];
                        classInfo.name = typ.FullName;
                        classInfoList.Add(classInfo);
                    }

                    var namespaceStructure = new Dictionary<string, List<ClassInfo>>();
                    foreach (var c in classInfoList)
                    {
                        var parentNamespace = string.Join(".",
                            c.name?.Split('.').Reverse().Skip(1).Reverse() ?? Array.Empty<string>());

                        if (!namespaceStructure.ContainsKey(parentNamespace))
                            namespaceStructure[parentNamespace] = [];

                        namespaceStructure[parentNamespace].Add(c);
                    }

                    var jsonString = JsonConvert.SerializeObject(namespaceStructure, Formatting.Indented);
                    File.WriteAllText(jsonPath, jsonString);
                    _logger?.LogInformation("Successfully summarized: {filePath}", dllFile.FullName);
                }
            }
            catch (Exception e)
            {
                _logger?.LogError(e.ToString());
            }
        }
    }


    internal class Options
    {
        [Option('d', "debug", Required = false, HelpText = "Enable verbose output.")]
        public bool Debug { get; set; } = false;

        [Option('i', "input", Required = true,
            HelpText = "Input file or directory. Ingested file types are `.cs`, `.dll`, and `.pdb`.")]
        public string InputFilePath { get; set; } = "";

        [Option('o', "output", Required = false, HelpText = "Output directory. (default `./.ast`)")]
        public string OutputDirectory { get; set; } = ".ast";

        [Option('e', "exclude", Required = false, HelpText = "Exclusion regex for while files to filter out.")]
        public string? ExclusionRegex { get; set; } = null;
    }
}
