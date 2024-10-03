# RubyAstGen

A Ruby parser than dumps the AST as JSON output. Uses the 
[`parser` gem](https://github.com/whitequark/parser/tree/90e0a4e2be86b02c423c77337adcfccdf6dd611b), thus supports
parsing Ruby version 1.8, 1.9, 2.0, 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 3.0, 3.1, and 3.2 syntax with 
backwards-compatible AST formats.

## Usage

```
usage: ruby_ast_gen [options]
    -i, --input    The input file or directory
    -o, --output   The output directory
    -e, --exclude  The exclusion regex
    -l, --log      The logging level
    --version      Print the version
    --help         Print usage
```

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake spec` to run the tests. You can 
also run `bin/console` for an interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`. To package, run `rake build`.

## License

The gem is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).
