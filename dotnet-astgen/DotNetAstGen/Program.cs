using System.Diagnostics;
using System.Text;
using System.Text.RegularExpressions;
using CommandLine;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.Extensions.Logging;
using Newtonsoft.Json;

namespace DotNetAstGen
{
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

                    _RunAstGet(
                        options.InputFilePath,
                        new DirectoryInfo(options.OutputDirectory),
                        options.ExclusionRegex);
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
    }


    internal class Options
    {
        [Option('d', "debug", Required = false, HelpText = "Enable verbose output.")]
        public bool Debug { get; set; } = false;

        [Option('i', "input", Required = true, HelpText = "Input file or directory.")]
        public string InputFilePath { get; set; } = "";

        [Option('o', "input", Required = false, HelpText = "Output directory. (default `./.ast`)")]
        public string OutputDirectory { get; set; } = ".ast";

        [Option('e', "exclude", Required = false, HelpText = "Exclusion regex for while files to filter out.")]
        public string? ExclusionRegex { get; set; } = null;
    }
}