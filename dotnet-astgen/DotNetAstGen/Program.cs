using System.Reflection;
using System.Text;
using CommandLine;
using DotNetAstGen.Utils;
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
                            .AddConsole()
                            .AddDebug();

                        if (options.Verbose)
                        {
                            builder.SetMinimumLevel(LogLevel.Debug);
                        }
                    });

                    _logger = LoggerFactory.CreateLogger<Program>();
                    _logger.LogDebug("Show verbose output.");

                    _RunAstGet(options.InputFilePath);
                });
        }

        private static void _RunAstGet(string inputPath)
        {
            if (Directory.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing directory {dirName}", inputPath);
                foreach (FileInfo fileInfo in new DirectoryInfo(inputPath).EnumerateFiles("*.cs"))
                {
                    _AstForFile(fileInfo);
                }
            }
            else if (File.Exists(inputPath))
            {
                _logger?.LogInformation("Parsing file {fileName}", inputPath);
                _AstForFile(new FileInfo(inputPath));
            }
            else
            {
                _logger?.LogError("The path {inputPath} does not exist!", inputPath);
                Environment.Exit(1);
            }
            _logger?.LogInformation("Parsing successful!");
        }

        private static void _AstForFile(FileInfo filePath)
        {
            var fullPath = filePath.FullName;
            _logger?.LogDebug("Parsing file: {filePath}", fullPath);
            using var streamReader = new StreamReader(fullPath, Encoding.UTF8);
            var programText = streamReader.ReadToEnd();
            var tree = CSharpSyntaxTree.ParseText(programText);
            _logger?.LogDebug("Successfully parsed: {filePath}", fullPath);
            var root = tree.GetCompilationUnitRoot();
            var rootType = root.GetType();
            IList<PropertyInfo> props = new List<PropertyInfo>(rootType.GetProperties());
            var jsonString = JsonConvert.SerializeObject(root, Formatting.Indented, new JsonSerializerSettings
            {
                ReferenceLoopHandling = ReferenceLoopHandling.Ignore,
                ContractResolver = new IgnorePropertiesResolver() // Comment this to see the unfiltered parser output
            });
            var outputName = Path.Combine(filePath.DirectoryName ?? "./", $"{Path.GetFileNameWithoutExtension(fullPath)}.json");
            _logger?.LogDebug("Writing AST to {astJsonPath}", outputName);
            File.WriteAllText(outputName, jsonString);
        }
    }


    internal class Options
    {
        [Option('v', "verbose", Required = false, HelpText = "Enable verbose output.")]
        public bool Verbose { get; set; }

        [Option('i', "input", Required = true, HelpText = "Input file or directory.")]
        public string InputFilePath { get; set; }
    }
}