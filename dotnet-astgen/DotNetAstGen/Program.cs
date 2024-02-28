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

                    if (options.InputFilePath != "") {
                        _RunAstGet(
                            options.InputFilePath,
                            new DirectoryInfo(options.OutputDirectory),
                            options.ExclusionRegex);
                    }

                    if (options.InputDLLFilePath != "") {
                        var dllName = Path.GetFileNameWithoutExtension(options.InputDLLFilePath);
                        int lastDotIndex = dllName.LastIndexOf('.');
                        var jsonName = lastDotIndex >= 0 ? Path.GetDirectoryName(options.InputDLLFilePath) + "\\" + dllName.Substring(lastDotIndex + 1) + ".json" : Path.GetDirectoryName(options.InputDLLFilePath) + "\\" + dllName + ".json";
                        if (options.OutputBuiltInJsonPath != "") 
                        {
                            jsonName = options.OutputBuiltInJsonPath;
                        }
                        ProcessDll(options.InputDLLFilePath, jsonName);
                    }

                });
        }

        private static void _RunAstGet(string inputPath, DirectoryInfo rootOutputPath, string? exclusionRegex)
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

            _logger?.LogInformation("AST generation complete");
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

        static void ProcessDll(string dllPath, string jsonPath)
        {
            var p = new ReaderParameters();
            p.ReadSymbols = true;

            var classInfoList = new List<ClassInfo>();

            using var x = AssemblyDefinition.ReadAssembly(dllPath, p);
            Regex typeFilter = new Regex("^(<PrivateImplementationDetails>|<Module>|.*AnonymousType|.*\\/).*", RegexOptions.IgnoreCase);
            Regex methodFilter = new Regex("^.*\\.(ctor|cctor)", RegexOptions.IgnoreCase);

            foreach (var typ in x.MainModule.GetAllTypes().DistinctBy(t => t.FullName).Where(t => t.Name != null).Where(t => !typeFilter.IsMatch(t.FullName)))
            {
                var classInfo = new ClassInfo();
                var methodInfoList = new List<MethodInfo>();

                foreach (var method in typ.Methods.Where(m => !methodFilter.IsMatch(m.Name)).Where( m => m.IsPublic))
                {
                    var methodInfo = new MethodInfo();
                    var parameterTypesList = new List<List<string>>();
                    methodInfo.name = method.Name;
                    methodInfo.returnType = method.ReturnType.ToString();
                    methodInfo.isStatic = method.IsStatic;
                    foreach (var param in method.Parameters)
                    {
                        parameterTypesList.Add([param.Name, param.ParameterType.FullName]);
                    }
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
                var parentNamespace = string.Join(".", c.name.Split('.').Reverse().Skip(1).Reverse());

                if (!namespaceStructure.ContainsKey(parentNamespace))
                    namespaceStructure[parentNamespace] = new List<ClassInfo>();

                namespaceStructure[parentNamespace].Add(c);
            }

            var jsonString = JsonConvert.SerializeObject(namespaceStructure, Formatting.Indented);
            File.WriteAllText(jsonPath, jsonString);
        }
    }


    internal class Options
    {
        [Option('d', "debug", Required = false, HelpText = "Enable verbose output.")]
        public bool Debug { get; set; } = false;

        [Option('i', "input", Required = false, HelpText = "Input file or directory.")]
        public string InputFilePath { get; set; } = "";

        [Option('o', "output", Required = false, HelpText = "Output directory. (default `./.ast`)")]
        public string OutputDirectory { get; set; } = ".ast";

        [Option('e', "exclude", Required = false, HelpText = "Exclusion regex for while files to filter out.")]
        public string? ExclusionRegex { get; set; } = null;

        [Option('l', "dll", Required = false, HelpText = "Input DLL file. Ensure a .pdb file is present of same name alongside DLL.")]
        public string InputDLLFilePath { get; set; } = "";

        [Option('b', "builtin", Required = false, HelpText = "The output JSON file. (default `./builtin_types.json`)")]
        public string OutputBuiltInJsonPath { get; set; } = "";
    }
}